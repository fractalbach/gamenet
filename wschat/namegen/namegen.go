// package namegen is for randomly generating usernames!
package namegen
//package main

import (
    //"fmt"
    "crypto/rand"
)


var adjectives = []string {
    "amazing",
    "thoughtful",
    "unsightly",
    "boundless",
    "dizzy",
    "fallacious",
    "impartial",
    "wasteful",
    "fantastic",
    "happy",
    "industrious",
    "diligent",
    "cuddly",
    "quarrelsome",
    "neighborly",
    "unusual",
    "adorable",
    "rustic",
    "flagrant",
    "quarrelsome",
    "fierce",
    "fearless",
    "rebel",
    "dangerous",
    "excellent",
    "glorious",
    "ad hoc",
    "swanky",
    "fresh",
    "electric",
    "delicious",
    "psychotic",
    "unruly",
    "unique",
    "defiant",
    "enchanting",
    "determined",
    "intelligent",
}




var nouns = []string {
    "mongoose",
    "porcupine",
    "marmalade",
    "jellyfish",
    "snail",
    "chicken",
    "rabbit",
    "ferret",
    "fawn",
    "mountain goat",
    "deer",
    "fox",
    "porpoise",
    "crocodile",
    "cat",
    "kitten",
    "starfish",
    "salamander",
    "chimpanzee",
    "jaguar",
    "skunk",
    "prairie dog",
    "cougar",
    "panda",
    "leopard",
    "coyote",
    "bear",
    "beaver",
    "giraffe",
    "hamster",
    "buffalo",
    "iguana",
    "finch",
    "sloth",
    "cheetah",
    "monkey",
    "snake",
    "canary",
    "alpaca",
    "lemur",
    "wolf",
    "mole",
    "vicuna",
    "oryx",
    "jackal",
    "highland cow",
    "dung beetle",
    "basilisk",
    "owl",
    "lion",
    "marten",
    "wombat",
    "elk",
    "warthog",
    "parrot",
    "marmoset",
    "moose",
    "chameleon",
    "anteater",
    "panda",
    "gazelle",
    "doe",
    "mink",
    "weasel",
    "jerboa",
    "opossum",
    "turtle",
    "tiger",
    "capybara",
    "chinchilla",
    "orangutan",
    "lynx",
    "rhinoceros",
    "gnu",
    "panther",
    "hare",
    "ibex",
    "mandrill",
}


// gimmeRandom creates 1 random byte from the crypto package,
// and then converts that into an integer within the maximum range [0, max].
func gimmeRandom(max int) int {
    a := make([]byte, 1)
    rand.Read(a)
    return int(a[0]) % max
}

func GenerateUsername() string {
    a := adjectives[gimmeRandom(len(adjectives)- 1)]
    n := nouns[gimmeRandom(len(nouns) - 1)]
    return a + " " + n 
}


// Note: the regular pseudo-random number gen would need a seed like this:
// rand.Seed(time.Now().UTC().UnixNano())


// func main() {
//     for i := 0; i < 10; i++ {
//         fmt.Println( GenerateUsername() )
//     }
// }
