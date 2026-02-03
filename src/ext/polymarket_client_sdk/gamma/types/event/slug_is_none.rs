use polymarket_client_sdk::gamma::types::response::Event;

pub fn slug_is_none(event: &Event) -> bool {
    event.slug.is_none()
}
