package views

type LoginRequest struct {
	Username string `form:"username"`
	Password string `form:"password"`
}
