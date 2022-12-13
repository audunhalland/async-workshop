# Rust async/web/"engineering" workshop
This workshop is an introduction to several Rust concepts/paradigms typically used when
developing _web services_. We'll look at a very simple Rust TODO application with a high level approach,
focus on its overall code structure/architecture, and how it feels like to:

* Read straight forward, high level Rust code
* Do some easy refactors
* Introduce a couple of new features.

It's not a tutorial for learning the _Rust language_. The existing code in our application
should look quite familiar to anyone coming from whatever modern programming language.

After the workshop, you should have some idea:

* How a Rust application is structured
* How `async/await` is used to achieve concurrency in web services
* How type-safe error handling looks (and works)
* How to write a unit test and an integration test in Rust
* How compile time checking of _basically everything_ can make a developer more productive
* About some of the tradeoffs we make when choosing a _runtime/GC_ language over Rust

# Preparations
1. [Install Rust](https://www.rust-lang.org/tools/install)
2. Install `docker` + `docker-compose`
3. Clone the workshop: `git clone https://github.com/audunhalland/async-workshop`
4. Use a suitable IDE
   * `vscode` + [Rust Analyzer](https://marketplace.visualstudio.com/items?itemName=matklad.rust-analyzer)
   * or alternatively, `CLion`/`IntelliJ Rust`
5. Install `sqlx` CLI: `cargo install sqlx-cli`
# Our TODO application
The main components of our application:

* A `PostgreSQL` database with one table, `todo_item`.
* A web server with a `GraphQL` endpoint for simple query and mutation of our TODO dataset.
* The `GraphQL Playground` in-browser app for easy interaction with our GraphQL schema.

GraphQL has been chosen over a REST interface. This was mainly done for simplicity: I didn't want
to focus on how to define web routes. How to define our application in terms of REST is left
as a bonus exercise.

# Exercise 1: Exploring the code structure
The Rust compilation unit is called a `crate`. All external libraries are crates.
Our TODO application is an _executable crate_, and its root module is defined in [src/main.rs](src/main.rs).

A rust project can be a _library_ instead of an executable. In that case, it would instead have a root module called [src/lib.rs](src/lib.rs).

However, this project has both `main.rs` and `lib.rs`! This means that it consists of _two crates_. A thin,
single-module crate containing the `fn main()` function that gets linked to the rest of the application, which is actually a _library_. The reason for this split is _integration tests_, which we will come back to.

You can take a look around, the code modules are as follows:

* [src/main.rs](src/main.rs) `fn main()`
* [src/lib.rs](src/lib.rs) `fn run()` + root module defining all other lib modules:
* [src/config.rs](src/config.rs) runtime config model needed to run the app
* [src/app.rs](src/app.rs) data structure for the running app
* [src/model.rs](src/model.rs) type definitions like `AppError` and `TodoFilter`
* [src/server.rs](src/server.rs) web server/endpoints
* [src/database.rs](src/database.rs) SQL queries
* [src/schema/](src/schema/) GraphQL schema definition and implementation
* [src/schema/query.rs](src/schema/query.rs) GraphQL queries (immutable)
* [src/schema/mutation.rs](src/schema/mutation.rs) GraphQL mutations (e.g. create a new TODO item)
* [src/schema/todo_item.rs](src/schema/todo_item.rs) _model_ for one TODO item
* [src/bus.rs](src/bus.rs) _event bus_ in case we have time! (client may subscribe to changes)
* [tests/repository_test.rs](tests/repository_test.rs) Integration tests interacting with a real PostgreSQL instance
* [.env](.env) `dotenv` file containing the URL to the database.

# Exercise 2: Run and pass the tests
As mentioned, this app depends on PostgreSQL, and we have integration tests. They need to communicate
with a real postgres instance. Let's start it:

```
$ docker-compose up -d
```

This should start a postgres server running at port `9876` (this may be changed by updating both `docker-compose.yml` and `.env`).

Next we'll try to run the tests:

```
$ cargo test
```

The compiler complains about the `sqlx::query_as!` macro. Our build depends on the DB schema
begin up to date. So we have to migrate the database:

```
$ cargo sqlx migrate run
Applied 1/migrate init (7.51911ms)
$ cargo test
```

All the tests, including the `src/` unit tests and the `tests/` integration tests should now pass.

# Excercise 3: Run the application and interact with _GraphQL Playground_
Now we're confident the application will work, we can run it:
```
$ cargo run
```

It will complain that there's no `DATABASE_URL` present. This is because, now in "production mode" we shouldn't use the `.env` file, but _real_ environemt variables. First define the variable:

```
$ export DATABASE_URL="postgres://rust:rust@localhost:9876/rust"
$ cargo run
```

The web server is hard coded to run at port `8000` (`main.rs`). Now, open a browser tab at [http://localhost:8000](http://localhost:8000).

We'll see an empty GraphQL query:

```graphql
{
}
```

Fill in a query to list our TODO items with the `description` field:

```graphql
{
    todoItems {
        description
        done
    }
}
```

and click the "execute" arrow. The response in the tab to the right should display:

```json
{
    "data": {
        "todoItems": []
    }
}
```

which means that a database query to fetch _no items_ has succeeded!

At last, we need to populate the database. Issue a `mutation` to the schema (recommended: do this in a separate _playground tab_, by clicking `[+]`):

```graphql
mutation {
    createTodoItem(description: "foobar") {
        id
        done
    }
}
```

This will return the `id` of the new item in the response. Querying `todoItems` again should indicate that we now have _one item_ in the database.

# Excercise 4: Actual hacking! Fetching a single TODO item
We'd like to imagine a web page that can display _one_ TODO item, given its `id`. But sadly, we only have a query to fetch a _list_ of items. Instead, we now want something like (GraphQL):

```
{
    todoItemById(id: "SOME_UUID") {
        ...
    }
}
```

This is not _wildly_ different from the existing `todoItems` query that returns an array. What's different here:

* It's a new query, because it has a different name
* The return type should be a single object instead of an array
* The return type should be able to represent failed attempt at finding something at this id

Using the knowledge of where the different code modules live and what they do, we can assume we need changes
to (at least) two different modules:

* `src/schema/query.rs`, containing GraphQL queries. The consumer-facing side.
* `src/repository.rs`, where database stuff actually happens.

We may attack this problem from either end first. You may prefer either one over the other. Being
good _TDD_ developers we'll implement the changes as two isolated tasks, with tests.

## Excercise 4.1: Create a new GraphQL query
Open `src/schema/query.rs`. In good TDD spirit we'll write the
test first. Scroll to the bottom section of the file, where the unit tests are defined (inside `mod tests { }`). Make a new test:

```rust
#[tokio::test]
async fn fetching_a_todo_item_by_id_should_work() {
    ...
}
```

or something like that. Inside, create a mock of our repo:

Issue an invocation of a GraphQL query:
```rust
let response = test_execute(
    mock(None),
    "
    {
        todoItemById(id: \"notsureyet\") {
            description
        }
    }
").await;
```

Note how `test_execute` is an `async fn`. Asynchronous functions implicitly return a `Future`. Futures may be `.await`ed inside
other async functions. Postfix `.await` is a syntax quite unique
to Rust, it was chosen because it reads more "naturally", and
is sequentially consistent with the order that code runs in.

The assertion should be similar to the assertion in the original test,
but without the `[]`, since we wanted to not have a list:

```rust
assert_eq!(
    response.data,
    value!({
        "todoItemById": {
            "description": "test"
        }
    })
);
```

`cargo test`. Compiling this should succeed. The test run should fail.

We need to implement the query `todoItemById`. Start by copying `async fn todo_items`. It should now be called `todo_item_by_id` (the GraphQL framework autogenerates the query). If you now `cargo test` again, you might get a different error message.

To make the test pass, we need two API changes:

1. Add an `id` parameter
2. Return an object instead of a list of objects

### Add a parameter
GraphQL functions look somewhat special. They have two arguments
that are not exposed through the schema: `&self` and `ctx`.

`&self` is the instance pointer to the `Query` struct. This struct
has no fields at all (except a `PhantomData`), so we can completely ignore it.
`ctx` is information about the current `GraphQL` context. For example,
we dig out the `Repository` instance using this context.

"real" GraphQL parameters are added after these two. Let's add the `id`:

```rust
async fn todo_item_by_id(
    &self,
    ctx: &async_graphql::Context<'_>,
    id: uuid::Uuid,
) -> ...
```

### Change the return type
Instead of `Vec` (Rust's growable array type), we'll use `Option`, the "nullability designator". Like so:

```rust
async fn todo_item_by_id(
    ...
) -> Result<Option<TodoItem>, AppError>
```

What's `Result`? It's an `enum` (sum type / tagged union) representing something that has either
succeeded or failed. It's first type parameter is the success type, the
second one is the failure type. Using this we get completely type-safe error
handling. Inspect the `AppError` type if you're curious about reason(s) this
method might fail.

> Note: You might also notice the _question mark operator_ after `.await?`. It simply means:
_If the preceding expression is an `Err`, the failure variant of `Result`, return it now_. It works a bit like `throw` in other languages.

We should get a compile error here:
```rust
Ok(todo_items)
```

`Ok` is the "success" variant of `Result`. But the _argument_ to `Ok` has the wrong type. It's expecting an `Option` but given a `Vec`.
We have to "unpack" this vector so that an empty vector becomes a `None` value, and a non-empty vector becomes a `Some` value (`None` and `Some(T)` are the two variants of the enum `Option<T>`).

We use the standard library for this conversion:
```rust
Ok(todo_items.into_iter().next())
```

i.e. first _convert_ the vector into an iterator, then try to extract
that iterator's "next value". The return type of `Iterator::next` is already `Option<T>`.

> Note: missing `return` keyword. The last expression in a Rust function
is that function's "returned" expression. Also note the missing trailing
semicolon, which indicates that this is an expression instead of a statement.

`cargo test`. It should complain about `"notsureyet"` not being parseable as an
`Uuid`. This should be proof that GraphQL recognizes our new query. To keep things
simple, just change it to `"000000000000-0000-0000-00000000"`.

### Dead simple mocking with [unimock](https://docs.rs/unimock/latest/unimock/)
`cargo test` again. This time it should fail rather miserably, with a `panic`. It should indicate some missing mock. We're missing a mock clause in our test, itself
proof that our new method was entered. Just copy `Unimock::new(..)` from the other test:

```rust
Unimock::new(
    database::ListTodoItemsMock
        .next_call(matching!(_))
        .returns(Ok(vec![test_todo_item()]))
),
```

```
$ cargo test
```

The tests should now run fine. But we forgot something. The mock clause has a _wildcard match_ (`_`) on the arguments passed to `database::list_todo_items`, and we do nothing with the `id` being passed to our function. `database` doesn't know anything about any `id`.

## Excercise 4.2: Extending Repository/SQL
Open `src/database.rs`. For the sake of DRY we can extend `async fn list_todo_items` to be able to optionally _filter_ on UUIDs (you are welcome to disagree with this design decision by calling it _unclean_ or whatever. But for now let's move on). This function accepts a `TodoFilter` as its first argument.

Open `src/model.rs`, where `TodoFilter` is defined:

```rust
#[derive(Debug)]
pub struct TodoFilter {
    /// Filter by a range (offset..size)
    pub range: Range<u32>,
}
```

There are some ways to add an optional filter here. It sounds like we might need `Option` again. Designing for the future, it is probably a good idea to support filtering for
_several_ UUIDs at the same time, like for a "batch-fetch" API call. For the sake of learning, let's try to add another field to `TodoFilter`:

```rust
    /// Optionally filter by TODO id(s)
    pub ids: Option<Vec<uuid::Uuid>>
```

When you now `cargo test` you'll get a couple of compile errors, so just insert `ids: None,` to the various `TodoFilter` constructors in the code base.

Now for some SQL changes. The query needs a dynamic-ish where clause:

```sql
SELECT id, description, done
FROM todo_item
OFFSET $1
LIMIT $2
```

> Note: As of `sqlx` version 0.5, only positional parameters are supported for PostgreSQL. It's a fairly new and upcoming library. But awesome: The `sqlx::query!` macro is a _compile time checked_ query, checked against the _running database_. In the upcoming version 0.6, sqlx will get some more powerful query syntax extension features, like named bind parameters.

We have to somehow bind `filter.ids` to the query, and we have to know how to write
a "conditional" where clause.

Let's try the "naive" thing first:

```sql
    WHERE id IN ($1)
```

The SQL `IN` operator is kind of special and weird. It accepts kind of a _syntactic_ list of values, so that calling this query with a one-sized and a two-sized vector is actually
two different queries. And matching "nothing" doesn't even make sense.

Instead we have to bind our `ids` as a _postgres array_. The SQL then would become

```sql
    WHERE id = any($1)
```

But still missing the "optionality", we can solve that by

```sql
    WHERE id = any($1) OR $1 IS NULL
```

We also need to actually bind `filter.ids` as the first parameter, so pass it
as the first one to the `query_as!` macro. You'll get a type error. `sqlx` expects
all _complex_ parameters as _references_ (i.e. _pointer_). We normally create
a reference to something by prefixing `&` in front of its expression. But that doesn't work here.
`ids` is a `Option<Vec<Uuid>>`. `sqlx` doesn't understand `Vec`. `Vec` is really an
implementation detail of how an array is laid out in memory: On the _heap_, because its length is dynamic. `sqlx` instead wants the type `&[Uuid]`, which is called
a _slice_ - an abstraction over an array exposing just a start pointer and a length,
with all other details completely hidden.

Normally, a `&some_vec` expression would _coerce_ (by ["Deref coercion"](https://doc.rust-lang.org/std/ops/trait.Deref.html#more-on-deref-coercion)) into a slice. But `&` only applies
to the _outer_ type: `&filter.ids` just becomes a `&Option<Vec<Uuid>>`. Because we instead want
the type `Option<&[Uuid]>`, that won't work. The standard library comes to the rescue again:
By using [Option::as_deref()](https://doc.rust-lang.org/std/option/enum.Option.html#method.as_deref) (this method _maps_ the mentioned deref coercion onto the type contained _inside_ the Option):

```rust
    filter.ids.as_deref(),
    filter.range.start as u32,
    filter.range.end as u32
```

## Excercise 4.3 writing an integration test
Let's hop into `tests/database_test.rs` and test if that SQL trick actually works.

Write a new test to prove that we are actually filtering by id.

At last, to finish all of exercise 4, make `todoItemById` _actually_ fetch a TODO item by `id`.

## Bonus challenge:
In the `Query::todo_item_by_id` _unit test_, find some way to assert that
the `id` argument actually gets passed correctly into `database`.

# Excercise 5: Structured logging
Rust has quite good support for flexible logging. When debugging a running service
we want detailed logs. We can use [tracing](https://docs.rs/tracing/) for structured
logging. This exercise demonstrates strucured logging with _instrumentation_.

Restart the application, now with `RUST_LOG=info cargo run`. There's already some
logging present, coming from the frameworks. Let's pretend we're interested
in logging that someone queried our `todoItemById`. The way to do this with tracing
is:

```rust
tracing::info!("todo_item_by_id called!");
```

Adding contextual info to log statements is done by "instrumenting" functions. An
instrumentation adds a "span" of context to all tracing statements issued from
functions called by the current function. For example:

```rust
#[tracing::instrument]
async fn foo() {}
```

By default, instrumenting a function will add the function's name and all parameters
to the `span`. Let's try to instrument `todo_item_by_id`.

```rust
#[tracing::instrument]
async fn todo_item_by_id(&self, ctx: ...) { ... }
```

There's a compile error, something about `ContextBase does not implement std::fmt::Debug`.

It's true. Not every Rust value may be printed to the terminal. You may have seen `#[derive(Debug)]`
before various `struct` definitions. This is how local types are provided with an implementation
of the [Debug trait](https://doc.rust-lang.org/std/fmt/trait.Debug.html), which means they become "printable". `ContextBase` is not a local
type, it comes from the `async_graphql` library. It does not implement `Debug`.
The easiest thing to do is to just skip types that we don't want tracing info for:

```rust
#[tracing::instrument(skip(ctx))]
async fn todo_item_by_id(&self, ctx: ..., id: uuid::Uuid)
```

... and now it should compile. Run again with `RUST_LOG=info cargo run`. Our trace statement
`todo_item_by_id` should be printed in the terminal when the query runs, along with the value of the `id` parameter.

The `Debug` trait is in fact a great way to protect sensitive information. Let's say
we had a password (implemented using the [newtype pattern](https://rust-unofficial.github.io/patterns/patterns/behavioural/newtype.html) / tuple struct):

```rust
struct Password(pub String);
```

Avoiding the `Debug` trait impl for this type will protect it from appearing in logs,
and that's statically guaranteed.

# Exercise 6: Subscriptions (the Ownership exercise)
There's a requirement that a frontend app needs to _subscribe_ to newly created TODO items. Let's
implement this very naively. The _GraphQL subscription_ subsystem has already been set up.

To test this in [GraphQL Playground](http://localhost:8000), create a new tab there and paste:

```
subscription {
    newTodoItems {
        description
        id
    }
}
```

Click the "play" icon, it should now display `"Listening..."` (it's using a websocket).

The task is to send a TODO item into `EventBus` after it has been created.

Find `src/mutation.rs` where items _get created_ in `create_todo_item`. After the repository has
"created" our item, it will return it and the next thing that happens is that it gets serialized
out of GraphQL as a direct response to the "creation" request.

Before the `async fn` returns, you must also send it into the event bus so that subscription(s) may pick it up.

We've been using `async_graphql`'s "object registry" before, with `::data_unchecked::<Repository>`. This
time we can just use `EventBus` instead of Repository (the objects get registered in `src/server.rs`).

`EventBus::sender` will get us a _sender_ that has a `::send(T)` method. "Behind" the scenes this is the
_sender end_ of a _broadcast channel_. A broadcast channel may have multiple senders and receivers.
The subscription connection instantiates a receiver, and stream data from that receiver out to websocket.

## The code
Write a statement that sends the object into the channel, and issue a `trace::error` if it didn't work.

[`Result` documentation](https://doc.rust-lang.org/std/result/enum.Result.html)

## Data race safety and ownership
Even if we write _asynchronous code_, it doesn't mean it's single threaded, like e.g. node.js. `tokio`
will spin up as many application threads as it sees fit. This means the the (ongoing) _subscription_ and _creation_
futures may run on different threads.

_Channels_ send "objects" around as a way for _threads_ to communicate. If some object is sent into a channel, it
must be _still alive_ and _valid_ when read out again (this is called _memory safety_ - i.e. the absence of dangling pointers etc that lead to undefined behaviour).

In many languages, data may be mutated freely, and this easily leads to data races, especially in
multithreaded scenarios. Let's say we push some _reference_ to an object into a channel. A "data-race-unsafe" language would permit mutation of this object after it has been posted to
the channel. The _sender_ thread may mutate it and the _receiver_ thread may also mutate it. This
is undefined behaviour.

Rust has an _auto trait_ (implicitly implemented trait) called `Send` that is automatically implemented for all
types that are safe to send between threads. Only `Send` types may be _sent_ into a channel.
Rust _references_ (`&T`) are `Send` (if `T` itself is), but _mutable_ references (`&mut T`) are not. What prevents us
from sending `&todo_item` into the channel? _Lifetimes_. Because it would likely be a dangling pointer
when read out from the other thread. The same applies to `&mut T` (doubly disallowed!). We _could_ use Rust's opt-in reference counting (`Arc<T>`) to make it work. But this would make the object immutable
forever, because shared references may not be mutated.

We're left with the most basic type of all, `T` (in this case, `TodoItem`). The _owned type_. The
_channel_ exclusively *owns* the object while it's in the channel, until it's read out, at which time
that ownership is transferred (and the object _moved_) to the function that reads it out. When programmed
in this way, it's safe to mutate the object in both threads. Because it's not one object, it's two.

Therefore the object must be `.clone()`d before it's sent into the channel.

## Further reading: Other reasons for _ownership_
A `Vec`, for example, points to internal heap memory allocation to an array. When such "objects" go
out of scope, we'd like that memory to be released. The compiler must issue explicit program instructions
for this "release of memory". In Rust parlance this is called "dropping" values:

```rust
fn foo() {
    let v = vec![1, 2, 3];
    // implicitly dropped here: `v` owns the heap memory, and the function owns `v` and now it goes out of scope.
}
```

We could do the same with two functions:

```rust
fn foo() {
    let v = vec![1, 2, 3];
    bar(v);
    // not dropped here! We passed ownership to `bar`. Double free is bad!
}

fn bar(v: Vec<u32>) {
    // `v` implicitly dropped here!
}
```

Or maybe send it into a channel:

```rust
fn foo() {
    let v = vec![1, 2, 3];
    bar(v);
}

fn bar(v: Vec<i32>) {
    some_channel.send(v);
    // not implicitly dropped anymore, because the channel now ows it.
}
```

Different language features comes into play at the same time to ensure statically safe programs.
