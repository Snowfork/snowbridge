---
layout: default
title: Scratch
permalink: /scratch/
---


```rust
use artemis_core::{
	registry::{App, lookup_app},
	AppId, Application, Message, Verifier,
};

fn dispatch(app: App, message: Message) -> DispatchResult {
    match app {
        App::ETH => T::AppETH::handle(message.payload),
        App::ERC20 => T::AppERC20::handle(message.payload)
    }
}
```

```js
// Javascript code with syntax highlighting.
var fun = function lang(l) {
  dateformat.i18n = require('./lang/' + l)
  return true;
}
```
