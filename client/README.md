## Game Client

If you are new to the game client, a good place to start examining the code is in client/src/main/kotlin/Core.kt, 
which is the entry point for the main game loop.


### Contents:

* client/src/ : Contains files related to the Game Client.
* client/src/main/ : files for building the production client.
* client/src/main/js/ : Contains javascript source files.
* client/src/main/kotlin/ : Contains kotlin source files.
* client/src/main/kotlin/events/ : Contains GameEvent class and classes extended from it.
* client/src/main/kotlin/lib/ : Contains related, but standalone kotlin source files (mostly math related currently)
* client/src/main/kotlin/objects/* : Contains GameObject and classes extended from it.
* client/src/main/resources/ : Contains resource files (and at least for the moment, misc files such as .html and .wasm)
* client/src/test/ : files for building and running tests on the client. File structure should resemble .../src/main/


### High-Level Structure:


![alt text](https://github.com/fractalbach/gamenet/blob/terrain/docs/open_structure.png "Logo Title Text 1")


### Game Loop Sequence:


![alt text](https://github.com/fractalbach/gamenet/blob/terrain/docs/open_update_sequence.png "Logo Title Text 1")
