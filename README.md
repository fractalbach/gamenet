# GAMENET

* Networking Server and Client for Multiplayer Games!

## Getting Started

These instructions will get you a copy of the project up and running on your local machine for development and testing purposes. See deployment for notes on how to deploy the project on a live system.


### Prerequisites

What things you need to install the software and how to install them

```
Give examples
```

### Installing

A step by step series of examples that tell you have to get a development env running

Say what the step will be

```
Give the example
```

And repeat

```
until finished
```

End with an example of getting some data out of the system or using it for a little demo

## Running the tests

Explain how to run the automated tests for this system

### Break down into end to end tests

Explain what these tests test and why

```
Give an example
```

### And coding style tests

Explain what these tests test and why

```
Give an example
```

## Deployment

Add additional notes about how to deploy this on a live system

## Built With

* Server side:
  * [The Go Programming Language](https://golang.org/)
  * [Gorilla web Toolkit](https://github.com/gorilla) - for Websockets in Go.
* Client side:
  * [Kotlin](https://kotlinlang.org/)   [(git)](https://github.com/JetBrains/kotlin)
  * [three.js](https://threejs.org/)    [(git)](https://github.com/mrdoob/three.js/)
  * [kotlin-math](https://github.com/romainguy/kotlin-math)
  * [uuid-random](https://github.com/jchook/uuid-random)


## Contributing

Please read [CONTRIBUTING.md], or just send pull request.

## Versioning

We use GitHub for versioning. For the versions available, see the [tags on this repository]

## Authors

* Just some guy, you know?
* industrious kitten.
* excellent owl.
* swanky lynx.
* dizzy chimpanzee.
* electric ferret.
* intelligent cougar.
* quarrelsome leopard.
* unusual beaver.
* neighborly iguana.
* fearless warthog.
* psychotic mongoose.
* dangerous gopher.

See also the list of [contributors] who participated in this project.

## License

This project is licensed under the MIT License - see the [LICENSE file](LICENSE) for details.

## Acknowledgments



|    Server        |     Client        |
| ---------------- | ----------------- |
| hey, LISTEN!     | hey, LISTEN!      |
|                  | <--- SYN          |
| SYN-ACK --->     |                   |
|                  | <--- ACK          |
| ESTABLISHED      | ESTABLISHED       |
| FIN --->         |                   |
|                  | <--- ACK          |
|                  | <--- FIN          |
| ACK --->         |                   |
| CLOSED           | CLOSED            |



---

Readme template from https://gist.github.com/PurpleBooth/109311bb0361f32d87a2