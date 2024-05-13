use futures::StreamExt;
use worker::*;

#[event(fetch)]
async fn main(req: Request, env: Env, _: Context) -> Result<Response> {
    if !req
        .headers()
        .get("Upgrade")?
        .is_some_and(|v| v == "websocket")
    {
        return Response::error("missing websocket upgrade header".to_string(), 400);
    }
    let WebSocketPair { client, server } = WebSocketPair::new()?;
    server.accept()?;

    wasm_bindgen_futures::spawn_local(async move {
        let mut stream = server.events().expect("can open stream");
        loop {
            match stream.next().await {
                Some(Ok(event)) => match event {
                    WebsocketEvent::Message(m) => {
                        if let Some(data) = m.text() {
                            console_debug!("data:{data:?}");
                            match server.send_with_str(data) {
                                Ok(_) => {}
                                Err(e) => console_debug!("err: {e}"),
                            }
                        }
                    }
                    WebsocketEvent::Close(e) => break,
                },
                Some(Err(e)) => console_debug!("err: {e}"),
                None => break,
            }
        }
    });
    Response::from_websocket(client)
}
