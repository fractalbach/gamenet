package main

import "bytes"


// hub maintains the set of active clients and broadcasts messages to the
// clients.
type Hub struct {
	// Registered clients.
	clients map[*Client]bool

	// Inbound messages from the clients.
	broadcast chan []byte

	// Register requests from the clients.
	register chan *Client

	// Unregister requests from clients.
	unregister chan *Client

	// Saved Messages
	q savedMessageQueue
}

func newHub() *Hub {
	return &Hub{
		broadcast:  make(chan []byte),
		register:   make(chan *Client),
		unregister: make(chan *Client),
		clients:    make(map[*Client]bool),
	}
}

func (h *Hub) run() {
	for {
		select {
		case client := <-h.register:
			h.clients[client] = true
			h.q.sendSavedMessages(client)

		case client := <-h.unregister:
			if _, ok := h.clients[client]; ok {
				delete(h.clients, client)
				close(client.send)
			}

		// Messages sent to the hub's broadcast channel,
		// are sent to all other active clients.  If a message is unable
		// to receive a broadcast message, that connection is dropped.
		case message := <-h.broadcast:
			h.q.add(message)
			for client := range h.clients {
				select {
				case client.send <- message:
				default:
					close(client.send)
					delete(h.clients, client)
				}
			}
		}
	}
}



const maxSave int = 30

// the first string in the array is the Most Recent message.
type savedMessageQueue struct {
	messages 	[maxSave][]byte
}


// 
// 		[1,2,3,4,5] --->  [2,3,4,5]  + [6]  --->  [2,3,4,5,6]
//
func (q *savedMessageQueue) add(msg []byte) {
	for i := 0; i < maxSave-1; i++ {
		q.messages[i] = q.messages[i+1]
	}
	q.messages[maxSave-1] = msg
}

func (q *savedMessageQueue) sendSavedMessages(c *Client) {
	for _, m := range q.messages {
		if (len(m) > 0) {
			m = bytes.Join([][]byte{[]byte("**"), m}, []byte(""))
			c.send <- m
		}
	}
}