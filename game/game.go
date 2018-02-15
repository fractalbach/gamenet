
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


func SendJsonToGame(gp *GamePram, jsonBlob []byte) {
    m := Message{}
    ParsePlayerMessage(jsonBlob, &m)
    gp.msgchan <-&m
}


// ParsePlayerMessage parses a json Message object into the specified variable.
func ParsePlayerMessage(jsonBlob []byte, m *Message) {
    if !json.Valid(jsonBlob) {
        log.Println("invalid json")
        return
    }
    err := json.Unmarshal(jsonBlob, m)
    if err != nil {
        log.Println("error:", err)
    }
}


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
}


// ______________________________________________________
//  Game Event Handler
// ------------------------------------------------------


func GameEventHandler(m *Message) {

    switch m.EventType {

    case "Move":
        return

    case "RequestGameState":
        return

    default:
        return
    }
}

func AdminEventHandler(a *AdminMessage) {
    switch a.Type {
    case "NewPlayer":
        return
    }
}

func AddEntity(w *World, e *Ent, id int) bool {
    if _, ok := w.Ents[id]; ok {
        return false
    }
    w.Ents[id] = e
    return true
}

func DeleteEntity(w *World, id int) {
    if _, ok := w.Ents[id]; ok {
        delete(w.Ents, id)
    }
}

func ChangeLocationEntity(w *World, id int, l []float64) {
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
    ok := AddEntity(w, &Ent{Name: username, Type: "player",}, id)
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