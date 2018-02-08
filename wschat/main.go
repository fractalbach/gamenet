package main

import (
	"flag"
	"log"
	"net/http"
	"time"
)


var addr = flag.String("addr", "localhost:8080", "http service address")



func main() {
	log.Println("Starting up Web Sockets Chat Server...")
	flag.Parse()

	log.Println("Starting Hub...")
	hub := newHub()
	go hub.run()


	/* 
	Create a Custom Server Multiplexer

	"servemux is an http request multiplexer. it matches the url of each 
	incoming request against a list of registered patterns and calls the
	handler for the pattern that most closely matches the url."
		https://golang.org/pkg/net/http/#ServeMux
	*/
	mux := http.NewServeMux()
	mux.HandleFunc("/", serveHome)
	mux.HandleFunc("/ws", func (w http.ResponseWriter, r *http.Request) {
		serveWs(hub, w, r)
	})


	// Define parameters for running a custom HTTP server
	s := &http.Server{
		Addr:           *addr,
		Handler:   		mux,
		ReadTimeout:    5 * time.Second,
		WriteTimeout:   5 * time.Second,
		MaxHeaderBytes: 1 << 20,
	}

	log.Println("Listening and Serving on ", *addr)
	log.Fatal(s.ListenAndServe())
}


/*
serveHome controls which files are accessible on the server based on how
the server responds to requests for those files.
*/
func serveHome(w http.ResponseWriter, r *http.Request) {
	log.Println(r.URL)
	if r.URL.Path != "/" {
		http.Error(w, "Not found", 404)
		return
	}
	if r.Method != "GET" {
		http.Error(w, "Method not allowed", 405)
		return
	}
	http.ServeFile(w, r, "home.html")
}




