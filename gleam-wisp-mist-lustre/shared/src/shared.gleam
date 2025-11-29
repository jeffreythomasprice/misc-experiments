import gleam/dynamic/decode
import gleam/json

pub type CurrentStateResponse {
  CurrentStateResponse(count: Int)
}

pub fn current_state_response_to_json(
  current_state_response: CurrentStateResponse,
) -> json.Json {
  let CurrentStateResponse(count:) = current_state_response
  json.object([
    #("count", json.int(count)),
  ])
}

pub fn current_state_response_decoder() -> decode.Decoder(CurrentStateResponse) {
  use count <- decode.field("count", decode.int)
  decode.success(CurrentStateResponse(count:))
}

pub type UpdateStateRequest {
  UpdateStateRequest(increment_by: Int)
}

pub fn update_state_request_to_json(
  update_state_request: UpdateStateRequest,
) -> json.Json {
  let UpdateStateRequest(increment_by:) = update_state_request
  json.object([
    #("increment_by", json.int(increment_by)),
  ])
}

pub fn update_state_request_decoder() -> decode.Decoder(UpdateStateRequest) {
  use increment_by <- decode.field("increment_by", decode.int)
  decode.success(UpdateStateRequest(increment_by:))
}
