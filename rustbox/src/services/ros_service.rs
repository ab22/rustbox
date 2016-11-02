use router_os;

struct RouterOSService<'a> {
	client: &'a router_os::Client;
}

impl<'a> RouterOSService {
	fn new(client: &'a router_os::Client) -> RouterOSService {
		RouterOSService {
			client: client,
		}
	}
}
