## `Bridge`
The Bridge is an application registry and handles the routing of messages from Substrate

### `constructor(address _verfierAddr, address[] _apps)` (public)
Initializes the Bridge and registers the applications

### `submit(address _appId, bytes _message)` (public)
Routes the message to the specified application ID after verifying the operator's signature

### `registerApp(address _appID)` (internal)
Registers a new application onto the bridge
