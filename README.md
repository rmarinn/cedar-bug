# `stacker::remaining_stack` Fails on Android Bindings

This repository demonstrates a bug where calling the function [`stacker::remaining_stack`](https://docs.rs/stacker/0.1.17/stacker/fn.remaining_stack.html) fails when run through Uniffi.

## Problem Overview

The function signature of `stacker::remaining_stack` is:

```rust
pub fn remaining_stack() -> Option<usize>
```

When called in native Rust (and probably the python bindings), the function works as expected and returns `Some(value)`. However, when the function is compiled to Kotlin via Uniffi, or possibly due to Android-specific limitations, it returns None.

## Example Usage in `cedar-policy`

In the `cedar-policy` project, the function `stacker::remaining_stack` is used in `cedar-policy-core` around line 941:

```rs
#[inline(always)]
fn stack_size_check() -> Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        // Error is caused by the left side always evaluating to 0 in 
        // the android binding
        if stacker::remaining_stack().unwrap_or(0) < REQUIRED_STACK_SPACE {
            return Err(EvaluationError::recursion_limit(None));
        }
    }
    Ok(())
}
```

If we follow up the chain, `stack_size_check` is called in `partial_interpret`.

```rs
// evaluator.rs
impl<'e> RestrictedEvaluator<'e> {
    pub fn partial_interpret(&self, expr: BorrowedRestrictedExpr<'_>) -> Result<PartialValue> {
        stack_size_check()?; // stack_size_check is called here

        let res = self.partial_interpret_internal(&expr);

        res.map(|pval| pval.with_maybe_source_loc(expr.source_loc().cloned()))
            .map_err(|err| match err.source_loc() {
                None => err.with_maybe_source_loc(expr.source_loc().cloned()),
                Some(_) => err,
            })
    }
}
```

Then `partial_interpret` is called in `Entity::new` -- which is the function we use to initialize entities.

```rs
// entity.rs
impl Entity {
    /// Create a new `Entity` with this UID, attributes, ancestors, and tags
    ///
    /// # Errors
    /// - Will error if any of the [`RestrictedExpr]`s in `attrs` or `tags` error when evaluated
    pub fn new(
        uid: EntityUID,
        attrs: impl IntoIterator<Item = (SmolStr, RestrictedExpr)>,
        ancestors: HashSet<EntityUID>,
        tags: impl IntoIterator<Item = (SmolStr, RestrictedExpr)>,
        extensions: &Extensions<'_>,
    ) -> Result<Self, EntityAttrEvaluationError> {
        let evaluator = RestrictedEvaluator::new(extensions);
        let evaluate_kvs = |(k, v): (SmolStr, RestrictedExpr), was_attr: bool| {
            let attr_val = evaluator
                .partial_interpret(v.as_borrowed()) // partial_interpret called here
                .map_err(|err| EntityAttrEvaluationError {
                    uid: uid.clone(),
                    attr_or_tag: k.clone(),
                    was_attr,
                    err,
                })?;
            Ok((k, attr_val.into()))
        };
        let evaluated_attrs = attrs
            .into_iter()
            .map(|kv| evaluate_kvs(kv, true))
            .collect::<Result<_, EntityAttrEvaluationError>>()?;
        let evaluated_tags = tags
            .into_iter()
            .map(|kv| evaluate_kvs(kv, false))
            .collect::<Result<_, EntityAttrEvaluationError>>()?;
        Ok(Entity {
            uid,
            attrs: evaluated_attrs,
            ancestors,
            tags: evaluated_tags,
        })
    }
}
```

## Steps to Reproduce

To reproduce the bug, follow these steps:

1. **Build the example code**: The Rust source code is located in the `./main/ directory`, while the Android binding code is located in the `./android_binding/src` directory.
2. **Kotlin Code**: The Kotlin code that uses the compiled bindings can be found in the `./android_binding/androidProj/app/src/main/java/com/example/androidproj/MainActivity.kt`.


## Building and running the example

The Rust source code can be found in `./main/` while the Android binding code can be found in `./android_binding/src`. The Kotlin code that uses the compiled bindings can be found in `./android_binding/androidProj/app/src/main/java/com/example/androidproj/MainActivity.kt`


## Prerequisites for Building

1. Install up `cargo-ndk` for cross-compiling:
```
cargo install cargo-ndk
```

2. Add targets for Android:
```
rustup target add \
        aarch64-linux-android \
        armv7-linux-androideabi \
        i686-linux-android \
        x86_64-linux-android
```

## Building the Android Binding

1. Build the rust code

```sh
cargo build
```

2. Use `cargo-ndk` to cross-compile the Rust library for Android

```sh
cargo ndk -o ./android_binding/androidProj/app/src/main/jniLibs --manifest-path ./Cargo.toml -t armeabi-v7a -t arm64-v8a -t x86 -t x86_64 build --release &&
```

3. Generate the Uniffi bindigs for Kotlin

```sh
cargo run --bin uniffi-bindgen generate --library ./target/debug/libmobile.so --language kotlin --out-dir ./android_binding/androidProj/app/src/main/java/com/example/rust_android
```

4. Open the `./android_binding/androidProj/` directory in Android Studio and run the project.
