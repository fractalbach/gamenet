// package pram is an abstraction of the Parallel Random Access Machine.
//
// Multiple concurrent Functions can be passed to the PRAM.  
// Each of those functions will be evaluated sequentially.
//
// Since all of the functions are evaluated within the PRAM's scope,
// Address Pointers should be used to modify data.
//
// In this example, a hash map is used to store pointers to Entity objects,
// Since hash maps do not support concurrent lookups, 
// the PRAM helps facilitate modified the hash map.
//
//          exampleEnts := map[int]*Ent{
//              1:  &Ent{"arthur dent", "human", []int{1, 2}},
//              2:  &Ent{"teapot", "item", []int{3, 4}},
//              3:  &Ent{"towel", "towel", []int{9, 5}},
//               4:  &Ent{"vogon", "alien", []int{30, 40}},
//              42: &Ent{"the ultimate answer", "unknown", []int{-100, -345}},
//          }
//          w := World{exampleEnts}
//
//          result := pram.read(func() interface{} {
//                  w.Ents[1].Location = []int{-48, -42}
//                  return *w.Ents[1].Location
//          })
//
// This function Wrote a new Location to the Entity object,
// and then Read back that value, placing it into the Result variable.
// Regardless of any other functions that may have been passed to the PRAM,
// the Result variable will be given the value []int{-48, -42}.
package pram

import (
    "encoding/json"
    "fmt"
    "log"
)

// ______________________________________________________
//  The PRAM
// ------------------------------------------------------

type PRAM struct {
    readerchan chan pramMessage
    writerchan chan funcMessage
    funchan    chan funMsg
}

func NewPRAM() *PRAM {
    storedpram := &PRAM{
        readerchan: make(chan pramMessage),
        writerchan: make(chan funcMessage),
        funchan:    make(chan funMsg),
    }
    go storedpram.run()
    return storedpram
}

// ______________________________________________________
//  Message Types
// ------------------------------------------------------

type pramMessage struct {
    returnChannel chan interface{}
    readerFun     (func() interface{})
}

type funcMessage struct {
    returnChannel chan interface{}
    evaluateFun   func()
}

type funMsg struct {
    returnChannel chan bool
    evaluateFun   func()
}

// ______________________________________________________
//  Running the PRAMA - Internal Operations
// ------------------------------------------------------

func (pram *PRAM) run() {
    for {
        select {
        case incomingRead := <-pram.readerchan:
            x := incomingRead.readerFun()
            incomingRead.returnChannel <- x

        case incomingWrite := <-pram.writerchan:
            incomingWrite.evaluateFun()

        case incomingFun := <-pram.funchan:
            incomingFun.evaluateFun()
            incomingFun.returnChannel <- true
        }
    }
}

// ______________________________________________________
//  Read/Write Functions with Interfaces
// ------------------------------------------------------

func (p *PRAM) read(f func() interface{}) interface{} {
    c := make(chan interface{})
    p.readerchan <- pramMessage{c, f}
    return <-c
}

func (p *PRAM) fun(f func()) bool {
    c := make(chan bool)
    p.funchan <- funMsg{c, f}
    return <-c
}

func fun(i interface{}, input interface{}) {
    print := fmt.Println
    switch i.(type) {

    case (func()):
        print("plain fun")
        i.(func())()

    case (func() interface{}):
        print("fun that returns stuff")
        i.(func() interface{})()

    case (func(interface{})):
        print("fun that takes stuff")
        //i.(func(interface{}))()

    case (func(interface{}) interface{}):
        print("fun that takes AND returns stuff" +
            "(but not always the same stuff!)")
        i.(func(interface{}) interface{})(input)
    }
}

// ______________________________________________________
//  Structures
// ------------------------------------------------------

type World struct {
    Ents map[int]*Ent
}

type Ent struct {
    Name     string
    Type     string
    Location []int
}

// ______________________________________________________
//  Private Functions
// ------------------------------------------------------

func stateAllEntities(w *World) []byte {
    b, err := json.Marshal(w.Ents)
    if err != nil {
        log.Println(err)
        return []byte{}
    }
    return b
}

// ______________________________________________________
//  Public Functions
// ------------------------------------------------------
func (pram *PRAM) GetFullGameStateJSON(w *World) []byte {
    result := pram.read(func() interface{} {
        return stateAllEntities(w)
    })
    return result.([]byte)
}

// ______________________________________________________
//  Main
// ------------------------------------------------------

func GenerateExampleWorld() World {
    return World{map[int]*Ent{
        1:  &Ent{"arthur dent", "human", []int{1, 2}},
        2:  &Ent{"teapot", "item", []int{3, 4}},
        3:  &Ent{"towel", "towel", []int{9, 5}},
        4:  &Ent{"vogon", "alien", []int{30, 40}},
        42: &Ent{"the ultimate answer", "unknown", []int{-100, -345}},
    }}
}

/*func main() {

    var print = fmt.Println

    exampleEnts := map[int]*Ent{
        1:  &Ent{"arthur dent", "human", []int{1, 2}},
        2:  &Ent{"teapot", "item", []int{3, 4}},
        3:  &Ent{"towel", "towel", []int{9, 5}},
        4:  &Ent{"vogon", "alien", []int{30, 40}},
        42: &Ent{"the ultimate answer", "unknown", []int{-100, -345}},
    }

    w := World{exampleEnts}

    pram := newPRAM()

    result := pram.read(func() interface{} {
        w.Ents[1].Location = []int{-48, -42}
        return *w.Ents[1]
    })
    print("result: ", result)

    pram.fun(func() {
        print(string(stateAllEntities(&w)))
    })
    time.Sleep(time.Second)
}*/
