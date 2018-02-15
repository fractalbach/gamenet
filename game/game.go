
package game


import (
    "encoding/json"
    "log"
)

const (
    dimensions = 3;
)

type World struct {
    Ents map[int]*Ent
    nextid int
}

type Ent struct {
    Name     string
    Type     string
    Location []float64
}

type AdminMessage struct {
    Type string
    Body interface{}
}

type Message struct {
    Type string
    Body interface{}
}

type AbstractEvent struct {

    EventId int
    EventType string

    SourceId int
    SourceType string

    TargetId int
    TargetType string

    Position []float64
}

// ______________________________________________________
//  Message Parse and Send
// ------------------------------------------------------


func PlayerJsonToEvent(jsonBlob []byte, playerId int) (*AbstractEvent, bool) {
    m := ParsePlayerMessage(jsonBlob)
    if e, ok := m.ConvertToEvent(playerId); ok {
        return e, true
    } else {
        // log.Println(playerId, "sent a message that can't become an event.")
        return &AbstractEvent{}, false
    }
}

// ParsePlayerMessage parses a json Message object into the specified variable.
func ParsePlayerMessage(jsonBlob []byte) *Message {
    m := &Message{}
    if !json.Valid(jsonBlob) {
        log.Println("Cannot Parse Player Message; Invalid Json")
        return m
    }
    err := json.Unmarshal(jsonBlob, m)
    if err != nil {
        log.Println("error UnMarshalling Message:", err)
    }
    return m
}


func (m *Message) ConvertToEvent(playerId int) (*AbstractEvent, bool) {
    switch m.Type {
    case "Move", "move":
        if a, ok := m.intoMoveEvent(); ok {
            a.SourceId = playerId
            return a, true
        }
    case "Login":
    }
    return &AbstractEvent{}, false
}


func (m *Message) intoMoveEvent() (*AbstractEvent, bool) {
    a := &AbstractEvent{EventType: "Move",}
    var pos []float64
    vals, ok := m.Body.([]interface{});
    if !ok {
        log.Println("Can't convert MOVE Message Body into []interface{}")
        return a, false
    }
    for _, v := range vals {
        if f, ok := v.(float64); ok {
            pos = append(pos, f)
        } else {
            log.Println("badly formatted MOVE Message; cannot convert to float")
            return a, false
        }
    }
    a.Position = pos
    return a, true
    
}


func (w *World) DoGameEvent(a *AbstractEvent) {
    switch a.EventType {
    case "Move":
        w.ChangeLocationEntity(a.SourceId, a.Position)
    case "Login":
    case "Logout":
    case "Create":
    case "Delete":
    }
}


func (w *World) AddEntity(e *Ent, id int) bool {
    if _, ok := w.Ents[id]; ok {
        return false
    }
    w.Ents[id] = e
    return true
}

func (w *World) DeleteEntity(id int) bool {
    if id == 0 {
        return false
    }
    if _, ok := w.Ents[id]; ok {
        delete(w.Ents, id)
        return true
    }
    return false
}

func (w *World) ChangeLocationEntity(id int, l []float64) {
    if _, ok := w.Ents[id]; ok {
        w.Ents[id].Location = l
    }
}

func (w *World) StateAllEntities() []byte {
    b, err := json.Marshal(w.Ents)
    if err != nil {
        log.Println(err)
        return []byte{}
    }
    return b
}



// ______________________________________________________
//  Main
// ------------------------------------------------------



func MakeNewWorld() *World {
    return &World {
        Ents: map[int]*Ent{},
        nextid: 1,
    }
}   

func (w *World) GeneratePlayer(username string) (int, bool) {
    id := w.GetNextId()
    ok := w.AddEntity(&Ent{Name: username, Type: "player",}, id)
    if ok {
        return id, true
    }
    return 0, false
}


func (w *World) GetNextId() int {
    output := w.nextid
    w.nextid++
    return output
}






/*
// ______________________________________________________
//  The PRAM
// ------------------------------------------------------

type GamePram struct {
    msgchan chan *Message
    adminchan chan *AdminMessage
}

func NewGamePram() *GamePram {
    storedpram := &GamePram{
        msgchan: make(chan *Message),
        adminchan: make(chan *AdminMessage),
    }
    go storedpram.run()
    return storedpram
}

func (gp *GamePram) run() {
    for {
        select {
        case incoming := <- gp.msgchan:
            GameEventHandler(incoming)

        case incoming := <- gp.adminchan:
            AdminEventHandler(incoming)
        }
    }
}*/



/*

func GetFullGameStateJSON(p *pram.PRAM, w *World) []byte {
    result := p.Read(func() interface{} {
        return stateAllEntities(w)
    })
    return result.([]byte)
}


*/

/*
func GenerateExampleWorld() World {
    return World{map[int]*Ent{
        1:   &Ent{"some name", "human", []float64{1, 2}},
        12:  &Ent{"some thing", "item", []float64{3, 4}},
        35:  &Ent{"towel", "towel", []float64{9, 5}},
        80:  &Ent{"maple tree", "tree", []float64{30, 40}},
    }}
}
*/