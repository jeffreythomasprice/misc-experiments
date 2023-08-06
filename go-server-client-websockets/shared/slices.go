package shared

func SetSliceLen[T any](s []T, desiredLen int) []T {
	if len(s) < desiredLen {
		return append(s, make([]T, desiredLen-len(s))...)
	} else if len(s) > desiredLen {
		return s[0:desiredLen]
	} else {
		return s
	}
}
