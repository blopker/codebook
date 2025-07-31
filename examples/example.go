package main

import myfmt "fmt"

type Userr struct {
	Namee string `json:"namme"`
}

type UserServicce interface {
	GetUserr(id string) Userr
}

const (
	MaxNameeSize = 100
)

func (u *Userr) GetUserr(prefixx string) Userr {
	return Userr{Namee: prefixx + "Alice"}
}

func main() {
	// I'm bad at speling alice
	myfmt.Println("Hello, Wolrd!")
	var alicz = "Alicz"
	myfmt.Println("Hellol, " + alicz)
	var rsvp = "RSVP"
	myfmt.Println("Hello, " + rsvp)
	cokbookkk := "test valie"
	myfmt.Println("Hello, " + cokbookkk)
outerr:
	for imdex := 0; imdex < 10; imdex++ {
		if imdex == 5 {
			break outerr
		}
	}
	itemns := []string{"firstt", "seconnd", "tihrd"}
	for indexx, valuue := range itemns {
		myfmt.Println(indexx, valuue)
	}
	myfmt.Println(itemns)
}
