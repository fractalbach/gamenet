// package namegen is for randomly generating usernames!
package namegen
//package main

import (
    //"fmt"
    "crypto/rand"
)



// syntactic categories
// syntactic categories such as noun phrase or verb phrase, 
// and combine these syntactic categories into trees representing the 
// phrase structure  of sentences: nested phrases, each marked with a category.
var consonant = []string {
    "bl", "br", "cl", "cr", "dr",
    "fr", "tr", "fl", "gl", "gr", "pl",
    "pr", "sl", "sm", "sp", "st",
    "b", "c", "d", "f",
    "g", "h", "j", "k", "l", "m", "n", "p", "q", "r", 
    "s", "t", "v","w", "x", "y", "z",
}

var vowel = []string {
    "a", "e", "i", "o", "u", 
    "ee", "ea", "ai", "oa", "igh",
    "ay", "ue", "ie", "ow", "oo", "ew",
    "ui", "ei", "y",
}


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
    "cantankerous",
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
    "platypus",
}


// gimmeRandom creates random bytes from the crypto package,
// and then converts that into an integer within the maximum range [0, max].
// NOTE:  0 < max < 255
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






// func generateTree() string {

// }


// Note: the regular pseudo-random number gen would need a seed like this:
// rand.Seed(time.Now().UTC().UnixNano())


// func main() {
//     for i := 0; i < 10; i++ {
//         fmt.Println( GenerateUsername() )
//     }
// }
