#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ResourceRequest {
    generation: u64,
}

impl ResourceRequest {
    pub fn generation(self) -> u64 {
        self.generation
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AsyncResourceState<T, E> {
    Idle,
    Loading {
        request: ResourceRequest,
        previous: Option<T>,
    },
    Ready {
        request: ResourceRequest,
        value: T,
    },
    Error {
        request: ResourceRequest,
        error: E,
        previous: Option<T>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AsyncResource<T, E = String> {
    generation: u64,
    state: AsyncResourceState<T, E>,
}

impl<T, E> Default for AsyncResource<T, E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, E> AsyncResource<T, E> {
    pub fn new() -> Self {
        Self {
            generation: 0,
            state: AsyncResourceState::Idle,
        }
    }

    pub fn ready(value: T) -> Self {
        let request = ResourceRequest { generation: 1 };
        Self {
            generation: request.generation,
            state: AsyncResourceState::Ready { request, value },
        }
    }

    pub fn failed(error: E) -> Self {
        let request = ResourceRequest { generation: 1 };
        Self {
            generation: request.generation,
            state: AsyncResourceState::Error {
                request,
                error,
                previous: None,
            },
        }
    }

    pub fn state(&self) -> &AsyncResourceState<T, E> {
        &self.state
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn current_request(&self) -> Option<ResourceRequest> {
        match &self.state {
            AsyncResourceState::Idle => None,
            AsyncResourceState::Loading { request, .. }
            | AsyncResourceState::Ready { request, .. }
            | AsyncResourceState::Error { request, .. } => Some(*request),
        }
    }

    pub fn begin_load(&mut self) -> ResourceRequest {
        let request = self.next_request();
        let previous = match std::mem::replace(&mut self.state, AsyncResourceState::Idle) {
            AsyncResourceState::Ready { value, .. } => Some(value),
            AsyncResourceState::Loading { previous, .. }
            | AsyncResourceState::Error { previous, .. } => previous,
            AsyncResourceState::Idle => None,
        };

        self.state = AsyncResourceState::Loading { request, previous };
        request
    }

    pub fn refresh(&mut self) -> ResourceRequest {
        self.begin_load()
    }

    pub fn resolve(&mut self, request: ResourceRequest, value: T) -> bool {
        if !self.accepts(request) {
            return false;
        }

        self.state = AsyncResourceState::Ready { request, value };
        true
    }

    pub fn reject(&mut self, request: ResourceRequest, error: E) -> bool {
        if !self.accepts(request) {
            return false;
        }

        let previous = match std::mem::replace(&mut self.state, AsyncResourceState::Idle) {
            AsyncResourceState::Loading { previous, .. } => previous,
            AsyncResourceState::Ready { value, .. } => Some(value),
            AsyncResourceState::Error { previous, .. } => previous,
            AsyncResourceState::Idle => None,
        };

        self.state = AsyncResourceState::Error {
            request,
            error,
            previous,
        };
        true
    }

    pub fn reset(&mut self) {
        self.generation = 0;
        self.state = AsyncResourceState::Idle;
    }

    pub fn is_idle(&self) -> bool {
        matches!(self.state, AsyncResourceState::Idle)
    }

    pub fn is_loading(&self) -> bool {
        matches!(self.state, AsyncResourceState::Loading { .. })
    }

    pub fn is_ready(&self) -> bool {
        matches!(self.state, AsyncResourceState::Ready { .. })
    }

    pub fn is_error(&self) -> bool {
        matches!(self.state, AsyncResourceState::Error { .. })
    }

    pub fn value(&self) -> Option<&T> {
        match &self.state {
            AsyncResourceState::Ready { value, .. } => Some(value),
            AsyncResourceState::Loading { previous, .. }
            | AsyncResourceState::Error { previous, .. } => previous.as_ref(),
            AsyncResourceState::Idle => None,
        }
    }

    pub fn ready_value(&self) -> Option<&T> {
        match &self.state {
            AsyncResourceState::Ready { value, .. } => Some(value),
            _ => None,
        }
    }

    pub fn error(&self) -> Option<&E> {
        match &self.state {
            AsyncResourceState::Error { error, .. } => Some(error),
            _ => None,
        }
    }

    pub fn accepts(&self, request: ResourceRequest) -> bool {
        matches!(
            &self.state,
            AsyncResourceState::Loading { request: current, .. } if *current == request
        )
    }

    fn next_request(&mut self) -> ResourceRequest {
        self.generation = next_generation(self.generation);
        ResourceRequest {
            generation: self.generation,
        }
    }
}

fn next_generation(current: u64) -> u64 {
    let next = current.wrapping_add(1);
    if next == 0 {
        1
    } else {
        next
    }
}

#[cfg(test)]
mod tests {
    use super::{AsyncResource, AsyncResourceState};

    #[test]
    fn new_resource_starts_idle() {
        let resource = AsyncResource::<String, String>::new();
        assert!(resource.is_idle());
        assert_eq!(resource.current_request(), None);
        assert_eq!(resource.value(), None);
    }

    #[test]
    fn begin_load_advances_generation_and_resolves_current_request() {
        let mut resource = AsyncResource::<String, String>::new();
        let request = resource.begin_load();

        assert!(resource.is_loading());
        assert_eq!(request.generation(), 1);
        assert!(resource.resolve(request, "ready".to_string()));
        assert!(resource.is_ready());
        assert_eq!(resource.ready_value().map(String::as_str), Some("ready"));
    }

    #[test]
    fn stale_responses_are_ignored() {
        let mut resource = AsyncResource::<String, String>::new();
        let stale = resource.begin_load();
        let current = resource.refresh();

        assert!(!resource.resolve(stale, "stale".to_string()));
        assert!(resource.resolve(current, "current".to_string()));
        assert_eq!(resource.ready_value().map(String::as_str), Some("current"));
    }

    #[test]
    fn refresh_preserves_previous_ready_value_while_loading() {
        let mut resource = AsyncResource::<String, String>::ready("cached".to_string());
        let request = resource.refresh();

        assert!(resource.is_loading());
        assert_eq!(resource.value().map(String::as_str), Some("cached"));
        assert!(resource.reject(request, "offline".to_string()));
        assert!(resource.is_error());
        assert_eq!(resource.error().map(String::as_str), Some("offline"));
        assert_eq!(resource.value().map(String::as_str), Some("cached"));
    }

    #[test]
    fn rejected_request_can_be_refreshed_with_new_generation() {
        let mut resource = AsyncResource::<String, String>::new();
        let failed = resource.begin_load();
        assert!(resource.reject(failed, "timeout".to_string()));
        let retry = resource.refresh();

        assert!(resource.is_loading());
        assert_ne!(failed, retry);
        assert!(resource.resolve(retry, "ok".to_string()));
        assert_eq!(resource.ready_value().map(String::as_str), Some("ok"));
    }

    #[test]
    fn reset_returns_resource_to_idle() {
        let mut resource = AsyncResource::<String, String>::ready("value".to_string());
        resource.reset();

        assert!(matches!(resource.state(), AsyncResourceState::Idle));
        assert_eq!(resource.generation(), 0);
    }
}
