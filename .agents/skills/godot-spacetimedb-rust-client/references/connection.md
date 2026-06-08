# Connection reminders

## Basic flow

- Build a `DbConnection` from the generated bindings.
- Provide the server URI and database name or identity.
- Register callbacks for connect, connect error, and disconnect.

## Runtime rule that bites people

- If the connection is not advanced, it will not process messages.
- In runtimes that do not auto-advance for you, subscriptions, reducer callbacks, and connection events will never fire.

## Client-data flow

- Subscribe to the rows the scene needs.
- Wait for the subscription-applied callback before trusting the local cache.
- Read from the local cache after the subscription is ready.

