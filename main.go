package main

import (
	"flag"
	"log"
	"net/http"
	"time"

	"github.com/fractalbach/gamenet/wschat"
)


var addr = flag.String("addr", "localhost:8080", "http service address")



func main() {
	log.Println("Starting up gamenet...")
	flag.Parse()

	log.Println("Starting Hub...")
	hub := wschat.NewHub()
	go hub.Run()


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
		wschat.ServeWs(hub, w, r)
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

	switch r.URL.Path {
	case "/": 		
		http.ServeFile(w, r, "website/gamechat.html")
		return

	case "/website/": 	
		http.ServeFile(w, r, "website/gamechat.html")
		return

	default: 		
		http.Error(w, "Not found", 404)
		return
	}

	if r.Method != "GET" {
		http.Error(w, "Method not allowed", 405)
		return
	}

	http.Error(w, "Bad Request.", 400)
}




