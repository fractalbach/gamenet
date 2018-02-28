# GAMENET

* Networking Server and Client for Multiplayer Games!

## Getting Started


### Prerequisites

JDK 8 or higher is required to run gradle. All other dependencies should be handled by gradle.
Dependencies downloaded by gradle are listed under "Built With".

### Installing

To build the project, run `$ ./gradlew build` in the project root directory.

## Running the tests

To run client QUnit tests, open out/test/client/client-tests.html in any relatively modern browser.

## Deployment

TODO: Add additional notes about how to deploy this on a live system

## Built With

* Server side:
  * [The Go Programming Language](https://golang.org/)
  * [Gorilla web Toolkit](https://github.com/gorilla) - for Websockets in Go.
* Client side:
  * [Kotlin](https://kotlinlang.org/)   [(git)](https://github.com/JetBrains/kotlin)
  * [three.js](https://threejs.org/)    [(git)](https://github.com/mrdoob/three.js/)
  * [WebAssemby](http://webassembly.org/) 
  * [emscripten](http://kripken.github.io/emscripten-site/) [(git)](https://github.com/kripken/emscripten)
  * [kotlin-math](https://github.com/romainguy/kotlin-math)
  * [uuid-random](https://github.com/jchook/uuid-random)


## Contributing

Pull requests are always welcome, particularly if they resolve any of the 
[current issues](https://github.com/fractalbach/gamenet/issues) .



## Versioning

We use GitHub for version control. Until basic usability is reached, no releases are being made.

## Authors

* Fractalbach
* TryExceptElse

See also the list of [contributors](https://github.com/fractalbach/gamenet/graphs/contributors) who participated in this project.

## License

This project is licensed under the MIT License - see the [LICENSE file](LICENSE) for details.
