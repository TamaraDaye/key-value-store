package store


// TODO: write various data encoding methods

type DataHeader struct {
	DataSize int
	filepointer int
}
type encoder interface {
	Encode(value string) []byte
	Decode(value string) string
}

type JsonEncoder struct{}

func (*JsonEncoder) Encoder(*Query) {
}

type BinaryEncoder struct{}


//TODO:
func (*BinaryEncoder) Encode(value string) []byte {
}
