package shared

const ReloadPath = "/_reload"

type WSMsgReloadClientToServer struct{}

type WSMsgReloadServerToClient struct {
	Key string
}
