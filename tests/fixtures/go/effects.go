package demo

//go:embed fixtures/*.txt
var content string

//go:generate stringer -type=Status

func Process(ch chan int) {
	go func() {
		ch <- 1
	}()
}
