package store

type encoder interface {
	Encode(*Query)
	Decode(*Query) string
}
