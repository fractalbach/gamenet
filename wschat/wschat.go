package wschat

import (
	"bytes"
	"log"
	"net/http"
	"time"
	"encoding/json"

	"github.com/fractalbach/gamenet/namegen"
	"github.com/gorilla/websocket"
)

const (
	// Time allowed to write a message to the peer.
	writeWait = 10 * time.Second

	// Time allowed to read the next pong message from the peer.
	pongWait = 60 * time.Second

	// Send pings to peer with this period. Must be less than pongWait.
	pingPeriod = (pongWait * 9) / 10

	// Maximum message size allowed from peer.
	maxMessageSize = 512

	// Maximum number of active clients allowed.
	maxActiveClients = 10

	// Number of Messages saved on the server.
	maxSave int = 30

)

var (
	newline = []byte{'\n'}
	space   = []byte{' '}
)

var upgrader = websocket.Upgrader{
	ReadBufferSize:  1024,
	WriteBufferSize: 1024,
}

// Client is a middleman between the websocket connection and the hub.
type Client struct {
	hub *Hub
	conn *websocket.Conn // The websocket connection.
	send chan []byte // Buffered channel of outbound messages.
	username string	// Username associated with a specific client.
}

// readPump pumps messages from the websocket connection to the hub.
//
// The application runs readPump in a per-connection goroutine. The application
// ensures that there is at most one reader on a connection by executing all
// reads from this goroutine.
func (c *Client) readPump() {
	defer func() {
		log.Println("Client Un-Registered: ", c.conn.RemoteAddr())
		c.hub.broadcast <- []byte("Goodbye, " + c.username + ".")
		c.hub.unregister <- c
		c.conn.Close()
	}()
	c.conn.SetReadLimit(maxMessageSize)
	c.conn.SetReadDeadline(time.Now().Add(pongWait))
	c.conn.SetPongHandler(func(string) error {
		c.conn.SetReadDeadline(time.Now().Add(pongWait)); 
		return nil 
	})
	for {
		_, message, err := c.conn.ReadMessage()
		if err != nil {
			if websocket.IsUnexpectedCloseError(err, websocket.CloseGoingAway, websocket.CloseAbnormalClosure) {
				log.Printf("error: %v", err)
			}
			break
		}
		message = bytes.TrimSpace(bytes.Replace(message, newline, space, -1))

		// Log the message before the additions, so you don't end up
		// with a bunch of duplicate timestamps and addresses in the log.
		log.Println(c.conn.RemoteAddr(), string(message))

		var jsonvalidator string
		if (json.Valid(message)) {
        	jsonvalidator = "Valid Json"
    	} else {
    		jsonvalidator = "INVALID JSON!"
    	}
		// Add a timestamp and IP address to the beginning of the message.
		// See: https://golang.org/pkg/bytes/#Join
		message = bytes.Join( [][]byte{  
			[]byte(prettyNow()), 
			[]byte(c.username),
			[]byte(jsonvalidator),
			message,
		}, []byte(" >> "));

		// Send the message to all other players.
		c.hub.broadcast <- message
	}
}

// writePump pumps messages from the hub to the websocket connection.
//
// A goroutine running writePump is started for each connection. The
// application ensures that there is at most one writer to a connection by
// executing all writes from this goroutine.
func (c *Client) writePump() {
	ticker := time.NewTicker(pingPeriod)
	defer func() {
		ticker.Stop()
		c.conn.Close()
	}()
	for {
		select {
		case message, ok := <-c.send:
			c.conn.SetWriteDeadline(time.Now().Add(writeWait))
			if !ok {
				// The hub closed the channel.
				c.conn.WriteMessage(websocket.CloseMessage, []byte{})
				return
			}

			w, err := c.conn.NextWriter(websocket.TextMessage)
			if err != nil {
				return
			}
			w.Write(message)

			// Add queued chat messages to the current websocket message.
			n := len(c.send)
			for i := 0; i < n; i++ {
				w.Write(newline)
				w.Write(<-c.send)
			}

			if err := w.Close(); err != nil {
				return
			}
		case <-ticker.C:
			c.conn.SetWriteDeadline(time.Now().Add(writeWait))
			if err := c.conn.WriteMessage(websocket.PingMessage, nil); err != nil {
				return
			}
		}
	}
}

// ServeWs handles websocket requests from the peer.
func ServeWs(hub *Hub, w http.ResponseWriter, r *http.Request) {

	// Check to see if there are too many active clients already.
	if thereAreTooManyActiveClients(hub, maxActiveClients) {
		log.Println("Too many active clients.")
		return
	}

	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		log.Println(err)
		return
	}

	// Register a new Client connection into the hub.
	client := &Client{
		hub: hub, 
		conn: conn, 
		send: make(chan []byte, 256),
		username: namegen.GenerateUsername(),
	}
	client.hub.register <- client
	log.Println(
		"Client Registered:", client.conn.RemoteAddr(), client.username)

	// Allow collection of memory referenced by the caller by doing all work in
	// new goroutines.
	go client.writePump()
	go client.readPump()

	client.hub.broadcast <- []byte("Welcome, "+client.username+".")
}














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

func NewHub() *Hub {
	return &Hub{
		broadcast:  make(chan []byte),
		register:   make(chan *Client),
		unregister: make(chan *Client),
		clients:    make(map[*Client]bool),
	}
}

func (h *Hub) Run() {
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




// the first string in the array is the Most Recent message.
type savedMessageQueue struct {
	messages 	[maxSave][]byte
}

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

func thereAreTooManyActiveClients(hub *Hub, max int) bool {
	return len(hub.clients) > max
}

// prettyNow returns a string with a human-readable time stamp.
// Useful for adding to messages.  for the day, use: "_2 Jan, "
func prettyNow() string {
	return time.Now().Format("3:04:05 PM")
}


func checkJsonMessage(incomingMessage []byte) bool {
    if !(json.Valid(incomingMessage)) {
        return true
    }
    return false
}


