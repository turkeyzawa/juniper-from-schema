//! This library contains a procedural macro that reads a GraphQL schema file, and generates the
//! corresponding [Juniper](https://crates.io/crates/juniper) [macro calls]. This means you can
//! have a real schema file and be guaranteed that it matches your Rust implementation. It also
//! removes most of the boilerplate involved in using Juniper.
//!
//! [macro calls]: https://graphql-rust.github.io/types/objects/complex_fields.html
//!
//! # Table of contents
//!
//! - [Example](#example)
//! - [Example web app](#example-web-app)
//! - [GraphQL features](#graphql-features)
//!     - [The `ID` type](#the-id-type)
//!     - [Custom scalar types](#custom-scalar-types)
//!     - [Special case scalars](#special-case-scalars)
//!     - [Interfaces](#interfaces)
//!     - [Union types](#union-types)
//!     - [Input objects](#input-objects)
//!     - [Enumeration types](#enumeration-types)
//!     - [Default argument values](#default-argument-values)
//! - [Supported schema directives](#supported-schema-directives)
//!     - [Definition for `@juniper`](#definition-for-juniper)
//!     - [Customizing ownership](#customizing-ownership)
//!     - [Infallible fields](#infallible-fields)
//! - [GraphQL to Rust types](#graphql-to-rust-types)
//! - [Query trails](#query-trails)
//!     - [Abbreviated example](#abbreviated-example)
//!     - [Types](#types)
//!     - [Downcasting for interface and union `QueryTrail`s](#downcasting-for-interface-and-union-querytrails)
//!     - [`QueryTrail`s for fields that take arguments](#querytrails-for-fields-that-take-arguments)
//! - [Customizing the error type](#customizing-the-error-type)
//! - [Customizing the context type](#customizing-the-context-type)
//! - [Inspecting the generated code](#inspecting-the-generated-code)
//!
//! # Example
//!
//! Schema:
//!
//! ```graphql
//! schema {
//!   query: Query
//!   mutation: Mutation
//! }
//!
//! type Query {
//!   // The directive makes the return value `FieldResult<String>`
//!   // rather than the default `FieldResult<&String>`
//!   helloWorld(name: String!): String! @juniper(ownership: "owned")
//! }
//!
//! type Mutation {
//!   noop: Boolean!
//! }
//! ```
//!
//! How you could implement that schema:
//!
//! ```
//! #[macro_use]
//! extern crate juniper;
//!
//! use juniper_from_schema::graphql_schema_from_file;
//!
//! // This is the important line
//! graphql_schema_from_file!("tests/schemas/doc_schema.graphql");
//!
//! pub struct Context;
//! impl juniper::Context for Context {}
//!
//! pub struct Query;
//!
//! impl QueryFields for Query {
//!     fn field_hello_world(
//!         &self,
//!         executor: &juniper::Executor<'_, Context>,
//!         name: String,
//!     ) -> juniper::FieldResult<String> {
//!         Ok(format!("Hello, {}!", name))
//!     }
//! }
//!
//! pub struct Mutation;
//!
//! impl MutationFields for Mutation {
//!     fn field_noop(&self, executor: &juniper::Executor<'_, Context>) -> juniper::FieldResult<&bool> {
//!         Ok(&true)
//!     }
//! }
//!
//! fn main() {
//!     let ctx = Context;
//!
//!     let query = "query { helloWorld(name: \"Ferris\") }";
//!
//!     let (result, errors) = juniper::execute(
//!         query,
//!         None,
//!         &Schema::new(Query, Mutation),
//!         &juniper::Variables::new(),
//!         &ctx,
//!     )
//!     .unwrap();
//!
//!     assert_eq!(errors.len(), 0);
//!     assert_eq!(
//!         result
//!             .as_object_value()
//!             .unwrap()
//!             .get_field_value("helloWorld")
//!             .unwrap()
//!             .as_scalar_value::<String>()
//!             .unwrap(),
//!         "Hello, Ferris!",
//!     );
//! }
//! ```
//!
//! And with `graphql_schema_from_file!` expanded your code would look something like this:
//!
//! ```
//! #[macro_use]
//! extern crate juniper;
//!
//! pub struct Context;
//! impl juniper::Context for Context {}
//!
//! pub struct Query;
//!
//! juniper::graphql_object!(Query: Context |&self| {
//!     field hello_world(&executor, name: String) -> juniper::FieldResult<String> {
//!         <Self as QueryFields>::field_hello_world(&self, &executor, name)
//!     }
//! });
//!
//! trait QueryFields {
//!     fn field_hello_world(
//!         &self,
//!         executor: &juniper::Executor<'_, Context>,
//!         name: String,
//!     ) -> juniper::FieldResult<String>;
//! }
//!
//! impl QueryFields for Query {
//!     fn field_hello_world(
//!         &self,
//!         executor: &juniper::Executor<'_, Context>,
//!         name: String,
//!     ) -> juniper::FieldResult<String> {
//!         Ok(format!("Hello, {}!", name))
//!     }
//! }
//!
//! pub struct Mutation;
//!
//! juniper::graphql_object!(Mutation: Context |&self| {
//!     field noop(&executor) -> juniper::FieldResult<&bool> {
//!         <Self as MutationFields>::field_noop(&self, &executor)
//!     }
//! });
//!
//! trait MutationFields {
//!     fn field_noop(&self, executor: &juniper::Executor<'_, Context>) -> juniper::FieldResult<&bool>;
//! }
//!
//! impl MutationFields for Mutation {
//!     fn field_noop(&self, executor: &juniper::Executor<'_, Context>) -> juniper::FieldResult<&bool> {
//!         Ok(&true)
//!     }
//! }
//!
//! type Schema = juniper::RootNode<'static, Query, Mutation>;
//!
//! fn main() {
//!     let ctx = Context;
//!
//!     let query = "query { helloWorld(name: \"Ferris\") }";
//!
//!     let (result, errors) = juniper::execute(
//!         query,
//!         None,
//!         &Schema::new(Query, Mutation),
//!         &juniper::Variables::new(),
//!         &ctx,
//!     )
//!     .unwrap();
//!
//!     assert_eq!(errors.len(), 0);
//!     assert_eq!(
//!         result
//!             .as_object_value()
//!             .unwrap()
//!             .get_field_value("helloWorld")
//!             .unwrap()
//!             .as_scalar_value::<String>()
//!             .unwrap(),
//!         "Hello, Ferris!",
//!     );
//! }
//! ```
//!
//! # Example web app
//!
//! You can find an example of how to use this library together with [Rocket] and [Diesel] to make
//! a GraphQL web app at <https://github.com/davidpdrsn/graphql-app-example> or an example of how
//! to use this library with [Actix] and [Diesel] at
//! <https://github.com/husseinraoouf/graphql-actix-example>.
//!
//! [Rocket]: https://rocket.rs
//! [Diesel]: http://diesel.rs
//! [Actix]: https://actix.rs/
//!
//! # GraphQL features
//!
//! The goal of this library is to support as much of GraphQL as Juniper does.
//!
//! Here is the complete list of features:
//!
//! Supported:
//! - Object types including converting lists and non-nulls to Rust types
//! - Custom scalar types including the `ID` type
//! - Interfaces
//! - Unions
//! - Input objects
//! - Enumeration types
//!
//! Not supported yet:
//! - Subscriptions (will be supported once Juniper supports subscriptions)
//! - Type extensions
//!
//! ## The `ID` type
//!
//! The `ID` GraphQL type will be generated into [`juniper::ID`].
//!
//! [`juniper::ID`]: https://docs.rs/juniper/latest/juniper/struct.ID.html
//!
//! ## Custom scalar types
//!
//! Custom scalar types will be generated into a newtype wrapper around a `String`. For example:
//!
//! ```graphql
//! scalar Cursor
//! ```
//!
//! Would result in
//!
//! ```
//! pub struct Cursor(pub String);
//! ```
//!
//! ## Special case scalars
//!
//! A couple of scalar names have special meaning. Those are:
//!
//! - `Url` becomes
//! [`url::Url`](https://docs.rs/url/2.1.0/url/struct.Url.html).
//! - `Uuid` becomes
//! [`uuid::Uuid`](https://docs.rs/uuid/0.7.4/uuid/struct.Uuid.html).
//! - `Date` becomes
//! [`chrono::naive::NaiveDate`](https://docs.rs/chrono/0.4.6/chrono/naive/struct.NaiveDate.html).
//! - `DateTimeUtc` becomes [`chrono::DateTime<chrono::offset::Utc>`] by default but if defined with
//! `scalar DateTimeUtc @juniper(with_time_zone: false)` it will become [`chrono::naive::NaiveDateTime`].
//!
//! Juniper doesn't support [`chrono::Date`](https://docs.rs/chrono/0.4.9/chrono/struct.Date.html)
//! so therefore this library cannot support that either. You can read about Juniper's supported
//! integrations [here](https://docs.rs/juniper/0.13.1/juniper/integrations/index.html).
//!
//! [`chrono::DateTime<chrono::offset::Utc>`]: https://docs.rs/chrono/0.4.9/chrono/struct.DateTime.html
//! [`chrono::naive::NaiveDateTime`]: https://docs.rs/chrono/0.4.9/chrono/naive/struct.NaiveDateTime.html
//!
//! ## Interfaces
//!
//! Juniper has several ways of representing GraphQL interfaces in Rust. They are listed
//! [here](https://graphql-rust.github.io/types/interfaces.html#enums) along with their advantages
//! and disadvantages.
//!
//! For the generated code we use the `enum` pattern because we found it to be the most flexible.
//!
//! Abbreviated example (find [complete example here](https://github.com/davidpdrsn/juniper-from-schema/blob/master/examples/interface.rs)):
//!
//! ```
//! # #[macro_use]
//! # extern crate juniper;
//! # use juniper::*;
//! # use juniper_from_schema::graphql_schema;
//! # fn main() {}
//! # pub struct Context;
//! # impl juniper::Context for Context {}
//! # pub struct Article { id: ID, text: String }
//! # impl ArticleFields for Article {
//! #     fn field_id(
//! #         &self,
//! #         executor: &Executor<'_, Context>,
//! #     ) -> FieldResult<&ID> { unimplemented!() }
//! #     fn field_text(
//! #         &self,
//! #         executor: &Executor<'_, Context>,
//! #     ) -> FieldResult<&String> { unimplemented!() }
//! # }
//! # pub struct Tweet { id: ID, text: String }
//! # impl TweetFields for Tweet {
//! #     fn field_id(
//! #         &self,
//! #         executor: &Executor<'_, Context>,
//! #     ) -> FieldResult<&ID> { unimplemented!() }
//! #     fn field_text(
//! #         &self,
//! #         executor: &Executor<'_, Context>,
//! #     ) -> FieldResult<&String> { unimplemented!() }
//! # }
//! #
//! graphql_schema! {
//!     schema {
//!         query: Query
//!     }
//!
//!     type Query {
//!         search(query: String!): [SearchResult!]! @juniper(ownership: "owned")
//!     }
//!
//!     interface SearchResult {
//!         id: ID!
//!         text: String!
//!     }
//!
//!     type Article implements SearchResult {
//!         id: ID!
//!         text: String!
//!     }
//!
//!     type Tweet implements SearchResult {
//!         id: ID!
//!         text: String!
//!     }
//! }
//!
//! pub struct Query;
//!
//! impl QueryFields for Query {
//!     fn field_search(
//!         &self,
//!         executor: &Executor<'_, Context>,
//!         trail: &QueryTrail<'_, SearchResult, juniper_from_schema::Walked>,
//!         query: String,
//!     ) -> FieldResult<Vec<SearchResult>> {
//!         let article: Article = Article { id: ID::new("1"), text: "Business".to_string() };
//!         let tweet: Tweet = Tweet { id: ID::new("2"), text: "1 weird tip".to_string() };
//!
//!         let posts = vec![
//!             SearchResult::from(article),
//!             SearchResult::from(tweet),
//!         ];
//!
//!         Ok(posts)
//!     }
//! }
//! ```
//!
//! The enum that gets generated has variants for each type that implements the interface and also
//! implements `From<T>` for each type.
//!
//! ## Union types
//!
//! Union types are basically just interfaces so they work in very much the same way.
//!
//! Abbreviated example (find [complete example here](https://github.com/davidpdrsn/juniper-from-schema/blob/master/examples/union_types.rs)):
//!
//! ```
//! # #[macro_use]
//! # extern crate juniper;
//! # use juniper::*;
//! # use juniper_from_schema::graphql_schema;
//! # fn main() {}
//! # pub struct Context;
//! # impl juniper::Context for Context {}
//! # pub struct Article { id: ID, text: String }
//! # impl ArticleFields for Article {
//! #     fn field_id(
//! #         &self,
//! #         executor: &Executor<'_, Context>,
//! #     ) -> FieldResult<&ID> { unimplemented!() }
//! #     fn field_text(
//! #         &self,
//! #         executor: &Executor<'_, Context>,
//! #     ) -> FieldResult<&String> { unimplemented!() }
//! # }
//! # pub struct Tweet { id: ID, text: String }
//! # impl TweetFields for Tweet {
//! #     fn field_id(
//! #         &self,
//! #         executor: &Executor<'_, Context>,
//! #     ) -> FieldResult<&ID> { unimplemented!() }
//! #     fn field_text(
//! #         &self,
//! #         executor: &Executor<'_, Context>,
//! #     ) -> FieldResult<&String> { unimplemented!() }
//! # }
//! #
//! graphql_schema! {
//!     schema {
//!         query: Query
//!     }
//!
//!     type Query {
//!         search(query: String!): [SearchResult!]! @juniper(ownership: "owned")
//!     }
//!
//!     union SearchResult = Article | Tweet
//!
//!     type Article {
//!         id: ID!
//!         text: String!
//!     }
//!
//!     type Tweet {
//!         id: ID!
//!         text: String!
//!     }
//! }
//!
//! pub struct Query;
//!
//! impl QueryFields for Query {
//!     fn field_search(
//!         &self,
//!         executor: &Executor<'_, Context>,
//!         trail: &QueryTrail<'_, SearchResult, juniper_from_schema::Walked>,
//!         query: String,
//!     ) -> FieldResult<Vec<SearchResult>> {
//!         let article: Article = Article { id: ID::new("1"), text: "Business".to_string() };
//!         let tweet: Tweet = Tweet { id: ID::new("2"), text: "1 weird tip".to_string() };
//!
//!         let posts = vec![
//!             SearchResult::from(article),
//!             SearchResult::from(tweet),
//!         ];
//!
//!         Ok(posts)
//!     }
//! }
//! ```
//!
//! ## Input objects
//!
//! Input objects will be converted into Rust structs with public fields.
//!
//! Abbreviated example (find [complete example here](https://github.com/davidpdrsn/juniper-from-schema/blob/master/examples/input_types.rs)):
//!
//! ```
//! # #[macro_use]
//! # extern crate juniper;
//! # use juniper::*;
//! # use juniper_from_schema::graphql_schema;
//! # fn main() {}
//! # pub struct Context;
//! # impl juniper::Context for Context {}
//! # pub struct Post { id: ID }
//! # impl PostFields for Post {
//! #     fn field_id(
//! #         &self,
//! #         executor: &Executor<'_, Context>,
//! #     ) -> FieldResult<&ID> {
//! #         unimplemented!()
//! #     }
//! #     fn field_title(
//! #         &self,
//! #         executor: &Executor<'_, Context>,
//! #     ) -> FieldResult<&String> {
//! #         unimplemented!()
//! #     }
//! # }
//! # pub struct Query;
//! # impl QueryFields for Query {
//! #     fn field_noop(
//! #         &self,
//! #         executor: &Executor<'_, Context>,
//! #     ) -> FieldResult<&bool> {
//! #         unimplemented!()
//! #     }
//! # }
//! graphql_schema! {
//!     schema {
//!         query: Query
//!         mutation: Mutation
//!     }
//!
//!     type Mutation {
//!         createPost(input: CreatePost!): Post @juniper(ownership: "owned")
//!     }
//!
//!     input CreatePost {
//!         title: String!
//!     }
//!
//!     type Post {
//!         id: ID!
//!         title: String!
//!     }
//!
//!     type Query { noop: Boolean! }
//! }
//!
//! pub struct Mutation;
//!
//! impl MutationFields for Mutation {
//!     fn field_create_post(
//!         &self,
//!         executor: &Executor<'_, Context>,
//!         trail: &QueryTrail<'_, Post, juniper_from_schema::Walked>,
//!         input: CreatePost,
//!     ) -> FieldResult<Option<Post>> {
//!         let title: String = input.title;
//!
//!         unimplemented!()
//!     }
//! }
//! ```
//!
//! From that example `CreatePost` will be defined as
//!
//! ```
//! pub struct CreatePost {
//!     pub title: String,
//! }
//! ```
//!
//! ## Enumeration types
//!
//! GraphQL enumeration types will be converted into normal Rust enums. The name of each variant
//! will be camel cased.
//!
//! Abbreviated example (find [complete example here](https://github.com/davidpdrsn/juniper-from-schema/blob/master/examples/enumeration_types.rs)):
//!
//! ```
//! # #[macro_use]
//! # extern crate juniper;
//! # use juniper::*;
//! # use juniper_from_schema::graphql_schema;
//! # fn main() {}
//! # pub struct Context;
//! # impl juniper::Context for Context {}
//! # pub struct Post { id: ID }
//! # impl PostFields for Post {
//! #     fn field_id(
//! #         &self,
//! #         executor: &Executor<'_, Context>,
//! #     ) -> FieldResult<&ID> {
//! #         Ok(&self.id)
//! #     }
//! # }
//! #
//! graphql_schema! {
//!     schema {
//!         query: Query
//!     }
//!
//!     enum Status {
//!         PUBLISHED
//!         UNPUBLISHED
//!     }
//!
//!     type Query {
//!         allPosts(status: Status!): [Post!]! @juniper(ownership: "owned")
//!     }
//!
//!     type Post {
//!         id: ID!
//!     }
//! }
//!
//! pub struct Query;
//!
//! impl QueryFields for Query {
//!     fn field_all_posts(
//!         &self,
//!         executor: &Executor<'_, Context>,
//!         trail: &QueryTrail<'_, Post, juniper_from_schema::Walked>,
//!         status: Status,
//!     ) -> FieldResult<Vec<Post>> {
//!         match status {
//!             Status::Published => unimplemented!("find published posts"),
//!             Status::Unpublished => unimplemented!("find unpublished posts"),
//!         }
//!     }
//! }
//! ```
//!
//! ## Default argument values
//!
//! In GraphQL you are able to provide default values for field arguments, provided the argument is
//! nullable.
//!
//! Arguments of the following types support default values:
//! - `Float`
//! - `Int`
//! - `String`
//! - `Boolean`
//! - Enumerations
//! - Input objects (as field arguments, see below)
//! - Lists containing some other supported type
//!
//! Abbreviated example (find [complete example here](https://github.com/davidpdrsn/juniper-from-schema/blob/master/examples/default_argument_values.rs)):
//!
//! ```
//! # #[macro_use]
//! # extern crate juniper;
//! # use juniper::*;
//! # use juniper_from_schema::graphql_schema;
//! # fn main() {}
//! # pub struct Context;
//! # impl juniper::Context for Context {}
//! # pub struct Post { id: ID }
//! # impl PostFields for Post {
//! #     fn field_id(
//! #         &self,
//! #         executor: &Executor<'_, Context>,
//! #     ) -> FieldResult<&ID> {
//! #         Ok(&self.id)
//! #     }
//! # }
//! #
//! graphql_schema! {
//!     schema {
//!         query: Query
//!     }
//!
//!     enum Status {
//!         PUBLISHED
//!         UNPUBLISHED
//!     }
//!
//!     input Pagination {
//!         pageSize: Int!
//!         cursor: ID
//!     }
//!
//!     type Query {
//!         allPosts(
//!             status: Status = PUBLISHED,
//!             pagination: Pagination = { pageSize: 20 }
//!         ): [Post!]! @juniper(ownership: "owned")
//!     }
//!
//!     type Post {
//!         id: ID!
//!     }
//! }
//!
//! pub struct Query;
//!
//! impl QueryFields for Query {
//!     fn field_all_posts(
//!         &self,
//!         executor: &Executor<'_, Context>,
//!         trail: &QueryTrail<'_, Post, juniper_from_schema::Walked>,
//!         status: Status,
//!         pagination: Pagination,
//!     ) -> FieldResult<Vec<Post>> {
//!         // `status` will be `Status::Published` if not given in the query
//!
//!         match status {
//!             Status::Published => unimplemented!("find published posts"),
//!             Status::Unpublished => unimplemented!("find unpublished posts"),
//!         }
//!     }
//! }
//! ```
//!
//! ### Input object gotchas
//!
//! Defaults for input objects are only supported as field arguments. The following is not
//! supported
//!
//! ```graphql
//! input SomeType {
//!   field: Int = 1
//! }
//! ```
//!
//! This isn't supported because [the spec is unclear about how to handle multiple nested
//! defaults](https://github.com/webonyx/graphql-php/issues/350).
//!
//! Also, defaults are only used if no arguments are passed. So given the schema
//!
//! ```graphql
//! input Input {
//!   a: String
//!   b: String
//! }
//!
//! type Query {
//!   field(arg: Input = { a: "a" }): Int!
//! }
//! ```
//!
//! and the query
//!
//! ```graphql
//! query MyQuery {
//!   field(arg: { b: "my b" })
//! }
//! ```
//!
//! The value of `arg` inside the resolver would be `Input { a: None, b: Some("my b") }`. Note that
//! even though `a` has a default value in the field doesn't get used here because we set `arg` in
//! the query.
//!
//! # Supported schema directives
//!
//! A number of [schema directives][] are supported that lets you customize the generated code:
//!
//! - `@juniper(ownership: "owned|borrowed|as_ref")`. For customizing ownership of returned data.
//! More info [here](#customizing-ownership).
//! - `@juniper(infallible: true|false)`. Customize if a field should return `Result<T, _>` or
//! just `T`. More info
//! [here](http://localhost:4000/juniper_from_schema/index.html#infallible-fields).
//! - `@deprecated`. For deprecating types in your schema. Also supports supplying a reason with
//! `@deprecated(reason: "...")`
//!
//! [schema directives]: https://www.apollographql.com/docs/apollo-server/schema/directives/
//!
//! ## Definition for `@juniper`
//!
//! Some tools that operate on your GraphQL schema require you to include the definition for all
//! directives used. So in case you need it the definition for `@juniper` is:
//!
//! ```graphql
//! directive @juniper(
//!     ownership: String = "borrowed",
//!     infallible: Boolean = false,
//!     with_time_zone: Boolean = true
//! ) on FIELD_DEFINITION
//! ```
//!
//! This directive definition is allowed in your schema, as well as any other directive definition.
//! Definitions of `@juniper` that differ from this are not allowed though.
//!
//! The definition might change in future versions. Please refer to the [changelog][].
//!
//! juniper-from-schema doesn't require to put this in your schema, so you only need to include it
//! if some other tool requires it.
//!
//! [changelog]: https://github.com/davidpdrsn/juniper-from-schema/blob/master/CHANGELOG.md
//!
//! ## Customizing ownership
//!
//! By default all fields return borrowed values. Specifically the type is
//! `juniper::FieldResult<&'a T>` where `'a` is the lifetime of `self`. This works well for
//! returning data owned by `self` and avoids needless `.clone()` calls you would need if fields
//! returned owned values.
//!
//! However if you need to change the ownership you have to add the directive
//! `@juniper(ownership:)` to the field in the schema.
//!
//! It takes the following arguments:
//!
//! - `@juniper(ownership: "borrowed")`: The data returned will be borrowed from `self`
//! (`FieldResult<&T>`).
//! - `@juniper(ownership: "owned")`: The return type will be owned (`FieldResult<T>`).
//! - `@juniper(ownership: "as_ref")`: Only applicable for `Option` and `Vec` return types. Changes
//! the inner type to be borrowed (`FieldResult<Option<&T>>` or `FieldResult<Vec<&T>>`).
//!
//! Example:
//!
//! ```
//! # #[macro_use]
//! # extern crate juniper;
//! # use juniper_from_schema::*;
//! # use juniper::*;
//! # pub struct Context;
//! # impl juniper::Context for Context {}
//! # fn main() {}
//! graphql_schema! {
//!     schema {
//!         query: Query
//!     }
//!
//!     type Query {
//!         borrowed: String!
//!         owned: String! @juniper(ownership: "owned")
//!         asRef: String @juniper(ownership: "as_ref")
//!     }
//! }
//!
//! pub struct Query;
//!
//! impl QueryFields for Query {
//!     fn field_borrowed(&self, _: &Executor<'_, Context>) -> FieldResult<&String> {
//!         // ...
//!         # unimplemented!()
//!     }
//!
//!     fn field_owned(&self, _: &Executor<'_, Context>) -> FieldResult<String> {
//!         // ...
//!         # unimplemented!()
//!     }
//!
//!     fn field_as_ref(&self, _: &Executor<'_, Context>) -> FieldResult<Option<&String>> {
//!         // ...
//!         # unimplemented!()
//!     }
//! }
//! ```
//!
//! All field arguments will be owned.
//!
//! ## Infallible fields
//!
//! By default the generated resolvers are fallible, meaining they return a `Result<T, _>` rather
//! than a bare `T`. You can customize that using `@juniper(infallible: true)`.
//!
//! Example:
//!
//! ```
//! # #[macro_use]
//! # extern crate juniper;
//! # use juniper_from_schema::*;
//! # use juniper::*;
//! # pub struct Context;
//! # impl juniper::Context for Context {}
//! # fn main() {}
//! graphql_schema! {
//!     schema {
//!         query: Query
//!     }
//!
//!     type Query {
//!         canError: String!
//!         cannotError: String! @juniper(infallible: true)
//!         cannotErrorAndOwned: String! @juniper(infallible: true, ownership: "owned")
//!     }
//! }
//!
//! pub struct Query;
//!
//! impl QueryFields for Query {
//!     fn field_can_error(&self, _: &Executor<'_, Context>) -> FieldResult<&String> {
//!         // ...
//!         # unimplemented!()
//!     }
//!
//!     fn field_cannot_error(&self, _: &Executor<'_, Context>) -> &String {
//!         // ...
//!         # unimplemented!()
//!     }
//!
//!     fn field_cannot_error_and_owned(&self, _: &Executor<'_, Context>) -> String {
//!         // ...
//!         # unimplemented!()
//!     }
//! }
//! ```
//!
//! # GraphQL to Rust types
//!
//! This is how the standard GraphQL types will be mapped to Rust:
//!
//! - `Int` -> `i32`
//! - `Float` -> `f64`
//! - `String` -> `String`
//! - `Boolean` -> `bool`
//! - `ID` -> [`juniper::ID`](https://docs.rs/juniper/latest/juniper/struct.ID.html)
//!
//! # Query trails
//!
//! If you're not careful about preloading associations for deeply nested queries you risk getting
//! lots of [N+1 query bugs][]. Juniper provides a [look ahead API][] which lets you inspect things
//! coming up further down a query. However the API is string based, so you risk making typos and
//! checking for fields that don't exist.
//!
//! `QueryTrail` is a thin wrapper around Juniper look aheads with generated methods for each field
//! on all your types. This means the compiler will reject your code if you're checking for invalid
//! fields.
//!
//! Resolver methods (`field_*`) that return object types (non scalar values) will also get a
//! `QueryTrail` argument besides the executor.
//!
//! Since the `QueryTrail` type itself is defined in this crate (rather than being inserted into
//! your code) we cannot directly add methods for your GraphQL fields. Those methods have to be
//! added through ["extension traits"](http://xion.io/post/code/rust-extension-traits.html). So if
//! you see an error like
//!
//! ```text
//!    |  trail.foo();
//!    |        ^^^ method not found in `&juniper_from_schema::QueryTrail<'a, User, juniper_from_schema::Walked>`
//!    |
//!    = help: items from traits can only be used if the trait is in scope
//! help: the following trait is implemented but not in scope, perhaps add a `use` for it:
//!    |
//! 2  | use crate::graphql_schema::query_trails::QueryTrailUserExtensions;
//!    |
//! ```
//!
//! Then adding `use crate::graphql_schema::query_trails::*` to you module should fix it. This is
//! necessary because all the extention traits are generated inside a module called `query_trails`.
//! This is done so you can glob import the `QueryTrail` extension traits without glob importing
//! everything from your GraphQL schema.
//!
//! If you just want everything from the schema `use crate::graphql_schema::*` will also bring in
//! the extension traits.
//!
//! [N+1 query bugs]: https://secure.phabricator.com/book/phabcontrib/article/n_plus_one/
//! [look ahead API]: https://docs.rs/juniper/0.11.1/juniper/struct.LookAheadSelection.html
//!
//! ## Abbreviated example
//!
//! Find [complete example here](https://github.com/davidpdrsn/juniper-from-schema/blob/master/examples/query_trails.rs)
//!
//! ```
//! # #[macro_use]
//! # extern crate juniper;
//! # use juniper::*;
//! # use juniper_from_schema::graphql_schema;
//! # fn main() {}
//! # pub struct Context;
//! # impl juniper::Context for Context {}
//! #
//! graphql_schema! {
//!     schema {
//!         query: Query
//!     }
//!
//!     type Query {
//!         allPosts: [Post!]! @juniper(ownership: "owned")
//!     }
//!
//!     type Post {
//!         id: Int!
//!         author: User!
//!     }
//!
//!     type User {
//!         id: Int!
//!     }
//! }
//!
//! pub struct Query;
//!
//! impl QueryFields for Query {
//!     fn field_all_posts(
//!         &self,
//!         executor: &Executor<'_, Context>,
//!         trail: &QueryTrail<'_, Post, juniper_from_schema::Walked>,
//!     ) -> FieldResult<Vec<Post>> {
//!         // Check if the query includes the author
//!         if let Some(_) = trail.author().walk() {
//!             // Somehow preload the users to avoid N+1 query bugs
//!             // Exactly how to do this depends on your setup
//!         }
//!
//!         // Normally this would come from the database
//!         let post = Post {
//!             id: 1,
//!             author: User { id: 1 },
//!         };
//!
//!         Ok(vec![post])
//!     }
//! }
//!
//! pub struct Post {
//!     id: i32,
//!     author: User,
//! }
//!
//! impl PostFields for Post {
//!     fn field_id(&self, executor: &Executor<'_, Context>) -> FieldResult<&i32> {
//!         Ok(&self.id)
//!     }
//!
//!     fn field_author(
//!         &self,
//!         executor: &Executor<'_, Context>,
//!         trail: &QueryTrail<'_, User, juniper_from_schema::Walked>,
//!     ) -> FieldResult<&User> {
//!         Ok(&self.author)
//!     }
//! }
//!
//! pub struct User {
//!     id: i32,
//! }
//!
//! impl UserFields for User {
//!     fn field_id(
//!         &self,
//!         executor: &Executor<'_, Context>,
//!     ) -> FieldResult<&i32> {
//!         Ok(&self.id)
//!     }
//! }
//! ```
//!
//! ## Types
//!
//! A query trail has two generic parameters: `QueryTrail<'a, T, K>`. `T` is the type the current
//! field returns and `K` is either `Walked` or `NotWalked`.
//!
//! The lifetime `'a` comes from Juniper and is the lifetime of the incoming query.
//!
//! ### `T`
//!
//! The `T` allows us to implement different methods for different types. For example in the
//! example above we implement `id` and `author` for `QueryTrail<'_, Post, K>` but only `id` for
//! `QueryTrail<'_, User, K>`.
//!
//! If your field returns a `Vec<T>` or `Option<T>` the given query trail will be `QueryTrail<'_,
//! T, _>`. So `Vec` or `Option` will be removed and you'll only be given the inner most type.
//! That is because in the GraphQL query syntax it doesn't matter if you're querying a `User`
//! or `[User]`. The fields you have access to are the same.
//!
//! ### `K`
//!
//! The `Walked` and `NotWalked` types are used to check if a given trail has been checked to
//! actually be part of a query. Calling any method on a `QueryTrail<'_, T, K>` will return
//! `QueryTrail<'_, T, NotWalked>`, and to check if the trail is actually part of the query you have
//! to call `.walk()` which returns `Option<QueryTrail<'_, T, Walked>>`. If that is a `Some(_)`
//! you'll know the trail is part of the query and you can do whatever preloading is necessary.
//!
//! Example:
//!
//! ```ignore
//! if let Some(walked_trail) = trail
//!     .some_field()
//!     .some_other_field()
//!     .third_field()
//!     .walk()
//! {
//!     // preload stuff
//! }
//! ```
//!
//! You can always run `cargo doc` and inspect all the methods on `QueryTrail` and in which
//! contexts you can call them.
//!
//! ## Downcasting for interface and union `QueryTrail`s
//!
//! _This section is mostly relevant if you're using
//! [juniper-eager-loading](https://crates.io/crates/juniper-eager-loading) however it isn't
//! specific to that library._
//!
//! If you have a `QueryTrail<'a, T, Walked>` where `T` is an interface or union type you can use
//! `.downcast()` to convert that `QueryTrail` into one of the implementors of the interface or
//! union.
//!
//! Example:
//!
//! ```
//! # #[macro_use]
//! # extern crate juniper;
//! # use juniper::*;
//! # use juniper_from_schema::graphql_schema;
//! # fn main() {}
//! # pub struct Context;
//! # impl juniper::Context for Context {}
//! # pub struct Article { id: ID }
//! # impl ArticleFields for Article {
//! #     fn field_id(
//! #         &self,
//! #         executor: &Executor<'_, Context>,
//! #     ) -> FieldResult<&ID> { unimplemented!() }
//! # }
//! # pub struct Tweet { id: ID, text: String }
//! # impl TweetFields for Tweet {
//! #     fn field_id(
//! #         &self,
//! #         executor: &Executor<'_, Context>,
//! #     ) -> FieldResult<&ID> { unimplemented!() }
//! # }
//! #
//! graphql_schema! {
//!     schema {
//!         query: Query
//!     }
//!
//!     type Query {
//!         search(query: String!): [SearchResult!]!
//!     }
//!
//!     interface SearchResult {
//!         id: ID!
//!     }
//!
//!     type Article implements SearchResult {
//!         id: ID!
//!     }
//!
//!     type Tweet implements SearchResult {
//!         id: ID!
//!     }
//! }
//!
//! pub struct Query;
//!
//! impl QueryFields for Query {
//!     fn field_search(
//!         &self,
//!         executor: &Executor<'_, Context>,
//!         trail: &QueryTrail<'_, SearchResult, juniper_from_schema::Walked>,
//!         query: String,
//!     ) -> FieldResult<&Vec<SearchResult>> {
//!         let article_trail: QueryTrail<'_, Article, Walked> = trail.downcast();
//!         let tweet_trail: QueryTrail<'_, Tweet, Walked> = trail.downcast();
//!
//!         // ...
//!         # unimplemented!()
//!     }
//! }
//! ```
//!
//! ### Why is this useful?
//!
//! If you were do perform some kind of preloading of data you might have a function that inspects
//! a `QueryTrail` and loads the necessary data from a database. Such a function could look like
//! this:
//!
//! ```ignore
//! fn preload_users(
//!     mut users: Vec<User>,
//!     query_trail: &QueryTrail<'_, User, Walked>,
//!     db: &Database,
//! ) -> Vec<User> {
//!     // ...
//! }
//! ```
//!
//! This function works well when we have field that returns `[User!]!`. That field is going to get
//! a `QueryTrail<'a, User, Walked>` which is exactly what `preload_users` needs.
//!
//! However, now imagine you have a schema like this:
//!
//! ```graphql
//! type Query {
//!     search(query: String!): [SearchResult!]!
//! }
//!
//! union SearchResult = User | City | Country
//!
//! type User {
//!     id: ID!
//!     city: City!
//! }
//!
//! type City {
//!     id: ID!
//!     country: Country!
//! }
//!
//! type Country {
//!     id: ID!
//! }
//! ```
//!
//! The method `QueryFields::field_search` will receive a `QueryTrail<'a, SearchResult, Walked>`.
//! That type doesn't work with `preload_users`. So we have to convert our `QueryTrail<'a,
//! SearchResult, Walked>` into `QueryTrail<'a, User, Walked>`.
//!
//! This can be done by calling `.downcast()` which automatically gets implemented for interface and
//! union query trails. See above for an example.
//!
//! ## `QueryTrail`s for fields that take arguments
//!
//! Sometimes you have GraphQL fields that take arguments that impact which things your resolvers
//! should return. `QueryTrail` therefore also allows you inspect arguments to fields.
//!
//! Abbreviated example:
//!
//! ```
//! # #[macro_use]
//! # extern crate juniper;
//! # use juniper_from_schema::*;
//! # pub struct Context;
//! # impl juniper::Context for Context {}
//! # fn main() {}
//! # pub struct Country {}
//! # impl CountryFields for Country {
//! #     fn field_users<'a>(
//! #         &self,
//! #         executor: &juniper::Executor<'a, Context>,
//! #         trail: &QueryTrail<'a, User, Walked>,
//! #         active_since: DateTime<Utc>,
//! #     ) -> juniper::FieldResult<Vec<User>> {
//! #         unimplemented!()
//! #     }
//! # }
//! # pub struct User {}
//! # impl UserFields for User {
//! #     fn field_id<'a>(
//! #         &self,
//! #         executor: &juniper::Executor<'a, Context>,
//! #     ) -> juniper::FieldResult<&juniper::ID> {
//! #         unimplemented!()
//! #     }
//! # }
//! use chrono::prelude::*;
//!
//! graphql_schema! {
//!     schema {
//!         query: Query
//!     }
//!
//!     type Query {
//!         countries: [Country!]! @juniper(ownership: "owned")
//!     }
//!
//!     type Country {
//!         users(activeSince: DateTimeUtc!): [User!]! @juniper(ownership: "owned")
//!     }
//!
//!     type User {
//!         id: ID!
//!     }
//!
//!     scalar DateTimeUtc
//! }
//!
//! pub struct Query;
//!
//! impl QueryFields for Query {
//!     fn field_countries<'a>(
//!         &self,
//!         executor: &'a juniper::Executor<'a, Context>,
//!         trail: &'a QueryTrail<'a, Country, Walked>
//!     ) -> juniper::FieldResult<Vec<Country>> {
//!         // Get struct that has all arguments passed to `Country.users`
//!         let args: CountryUsersArgs<'a> = trail.users_args();
//!
//!         // The struct has methods for each argument, e.g. `active_since`.
//!         //
//!         // Notice that it automatically converts the incoming value to
//!         // a `DateTime<Utc>`.
//!         let _: Option<DateTime<Utc>> = args.active_since();
//!
//!         # unimplemented!()
//!         // ...
//!     }
//! }
//! ```
//!
//! You can also elide the `'a` lifetime:
//!
//! ```
//! # #[macro_use]
//! # extern crate juniper;
//! # use juniper_from_schema::*;
//! # pub struct Context;
//! # impl juniper::Context for Context {}
//! # fn main() {}
//! # pub struct Country {}
//! # impl CountryFields for Country {
//! #     fn field_users<'a>(
//! #         &self,
//! #         executor: &juniper::Executor<'a, Context>,
//! #         trail: &QueryTrail<'a, User, Walked>,
//! #         active_since: DateTime<Utc>,
//! #     ) -> juniper::FieldResult<Vec<User>> {
//! #         unimplemented!()
//! #     }
//! # }
//! # pub struct User {}
//! # impl UserFields for User {
//! #     fn field_id<'a>(
//! #         &self,
//! #         executor: &juniper::Executor<'a, Context>,
//! #     ) -> juniper::FieldResult<&juniper::ID> {
//! #         unimplemented!()
//! #     }
//! # }
//! # use chrono::prelude::*;
//! # graphql_schema! {
//! #     schema {
//! #         query: Query
//! #     }
//! #     type Query {
//! #         countries: [Country!]! @juniper(ownership: "owned")
//! #     }
//! #     type Country {
//! #         users(activeSince: DateTimeUtc!): [User!]! @juniper(ownership: "owned")
//! #     }
//! #     type User {
//! #         id: ID!
//! #     }
//! #     scalar DateTimeUtc
//! # }
//! # pub struct Query;
//! #
//! impl QueryFields for Query {
//!     fn field_countries(
//!         &self,
//!         executor: &juniper::Executor<'_, Context>,
//!         trail: &QueryTrail<'_, Country, Walked>
//!     ) -> juniper::FieldResult<Vec<Country>> {
//!         let args: CountryUsersArgs = trail.users_args();
//!
//!         # unimplemented!()
//!         // ...
//!     }
//! }
//! ```
//!
//! The name of the arguments struct will always be `{name of type}{name of field}Args` (e.g.
//! `CountryUsersArgs`). The method names will always be the name of the arguments in snake case.
//!
//! The `*_args` method is only defined on `Walked` query trails so if you get an error like:
//!
//! ```text
//! ---- src/lib.rs -  (line 10) stdout ----
//! error[E0599]: no method named `users_args` found for type `&QueryTrail<'_, Country, Walked>` in the current
//!  scope
//!   --> src/lib.rs:10:1
//!    |
//! 10 |         trail.users_args();
//!    |               ^^^^^^^^^^^^ method not found in `&QueryTrail<'_, Country, Walked>`
//! ```
//!
//! It is likely because you've forgotten to call [`.walk()`][] on `trail`.
//!
//! [`.walk()`]: struct.QueryTrail.html#method.walk
//!
//! Remember that you can always run `cargo doc` to get a high level overview of the generated
//! code.
//!
//! # Customizing the error type
//!
//! By default the return type of the generated field methods will be [`juniper::FieldResult<T>`].
//! That is just a type alias for `std::result::Result<T, juniper::FieldError>`. Should you want to
//! use a different error type than [`juniper::FieldError`] that can be done by passing `,
//! error_type: YourType` to [`graphql_schema_from_file!`].
//!
//! Just keep in that your custom error type must implement [`juniper::IntoFieldError`] to
//! type check.
//!
//! Example:
//!
//! ```
//! # #[macro_use]
//! # extern crate juniper;
//! # use juniper::*;
//! # use juniper_from_schema::graphql_schema_from_file;
//! # fn main() {}
//! # pub struct Context;
//! # impl juniper::Context for Context {}
//! # pub struct Mutation;
//! # impl MutationFields for Mutation {
//! #     fn field_noop(&self, executor: &Executor<'_, Context>) -> Result<&bool, MyError> {
//! #         Ok(&true)
//! #     }
//! # }
//! graphql_schema_from_file!("tests/schemas/doc_schema.graphql", error_type: MyError);
//!
//! pub struct MyError(String);
//!
//! impl juniper::IntoFieldError for MyError {
//!     fn into_field_error(self) -> juniper::FieldError {
//!         // Perform custom error handling
//!         juniper::FieldError::from(self.0)
//!     }
//! }
//!
//! pub struct Query;
//!
//! impl QueryFields for Query {
//!     fn field_hello_world(
//!         &self,
//!         executor: &Executor<'_, Context>,
//!         name: String,
//!     ) -> Result<String, MyError> {
//!         Ok(format!("Hello, {}!", name))
//!     }
//! }
//! ```
//!
//! [`graphql_schema!`] does not support changing the error type.
//!
//! [`graphql_schema!`]: macro.graphql_schema.html
//! [`graphql_schema_from_file!`]: macro.graphql_schema_from_file.html
//! [`juniper::IntoFieldError`]: https://docs.rs/juniper/0.11.1/juniper/trait.IntoFieldError.html
//! [`juniper::FieldError`]: https://docs.rs/juniper/0.11.1/juniper/struct.FieldError.html
//! [`juniper::FieldResult<T>`]: https://docs.rs/juniper/0.11.1/juniper/type.FieldResult.html
//!
//! # Customizing the context type
//!
//! By default the generate code will assume your context type is called `Context`. If that is not
//! the case you can customize it by calling [`graphql_schema_from_file!`] with `context_type: NewName`.
//!
//! Example:
//!
//! ```
//! # #[macro_use]
//! # extern crate juniper;
//! # use juniper::*;
//! # use juniper_from_schema::graphql_schema_from_file;
//! # fn main() {}
//! # pub struct Mutation;
//! # impl MutationFields for Mutation {
//! #     fn field_noop(&self, executor: &Executor<'_, MyContext>) -> juniper::FieldResult<&bool> {
//! #         Ok(&true)
//! #     }
//! # }
//! graphql_schema_from_file!("tests/schemas/doc_schema.graphql", context_type: MyContext);
//!
//! pub struct MyContext;
//! impl juniper::Context for MyContext {}
//!
//! pub struct Query;
//!
//! impl QueryFields for Query {
//!     fn field_hello_world(
//!         &self,
//!         executor: &Executor<'_, MyContext>,
//!         name: String,
//!     ) -> juniper::FieldResult<String> {
//!         Ok(format!("Hello, {}!", name))
//!     }
//! }
//! ```
//!
//! [`graphql_schema!`] does not support changing the context type.
//!
//! [`graphql_schema!`]: macro.graphql_schema.html
//! [`graphql_schema_from_file!`]: macro.graphql_schema_from_file.html
//!
//! # Inspecting the generated code
//!
//! If you wish to see exactly what code gets generated you can set the env var
//! `JUNIPER_FROM_SCHEMA_DEBUG` to `1` when compiling. For example:
//!
//! ```bash
//! JUNIPER_FROM_SCHEMA_DEBUG=1 cargo build
//! ```
//!
//! The code will not be formatted so it might be tricky to read. The easiest way to fix this is to
//! copy the printed code to a file and run it through [rustfmt].
//!
//! Alternatively you can include the [feature] called `"format-debug-output"`. This will run the
//! output through [rustfmt] before printing it. That way you don't have to do that manually.
//! Example `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! juniper-from-schema = { version = "x.y.z", features = ["format-debug-output"] }
//! ```
//!
//! Unfortunately this requires that you're using nightly, because [rustfmt requires
//! nightly](https://github.com/rust-lang/rustfmt#installing-from-source). It might also break your
//! build because [rustfmt] doesn't always compile for some reason ¯\\\_(ツ)\_/¯. If you experience
//! this just remove the `"format-debug-output"` feature and format the output manually.
//!
//! [feature]: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#choosing-features
//! [rustfmt]: https://github.com/rust-lang/rustfmt

#![deny(
    missing_docs,
    unused_imports,
    dead_code,
    unused_variables,
    unused_must_use
)]
#![doc(html_root_url = "https://docs.rs/juniper-from-schema/0.5.2")]

use juniper::{DefaultScalarValue, LookAheadSelection};
use std::marker::PhantomData;

pub use juniper_from_schema_code_gen::{graphql_schema, graphql_schema_from_file};

/// A type used to parameterize `QueryTrail` to know that `walk` has been called.
pub struct Walked;

/// A type used to parameterize `QueryTrail` to know that `walk` has *not* been called.
pub struct NotWalked;

/// A wrapper around a `juniper::LookAheadSelection` with methods for each possible child.
pub struct QueryTrail<'a, T, K> {
    #[doc(hidden)]
    pub look_ahead: Option<&'a LookAheadSelection<'a, DefaultScalarValue>>,
    #[doc(hidden)]
    pub node_type: PhantomData<T>,
    #[doc(hidden)]
    pub walked: K,
}

impl<'a, T> QueryTrail<'a, T, NotWalked> {
    /// Check if the trail is present in the query being executed
    pub fn walk(self) -> Option<QueryTrail<'a, T, Walked>> {
        match self.look_ahead {
            Some(inner) => Some(QueryTrail {
                look_ahead: Some(inner),
                node_type: self.node_type,
                walked: Walked,
            }),
            None => None,
        }
    }
}

impl<'a, T, K> QueryTrail<'a, T, K> {
    #[allow(clippy::new_ret_no_self)]
    #[doc(hidden)]
    #[allow(missing_docs)]
    pub fn new(lh: &'a LookAheadSelection<'a, DefaultScalarValue>) -> QueryTrail<'a, T, Walked> {
        QueryTrail {
            look_ahead: Some(lh),
            node_type: PhantomData,
            walked: Walked,
        }
    }
}

#[cfg(test)]
mod test {
    #[allow(unused_imports)]
    use super::*;

    #[rustversion::nightly]
    #[test]
    fn test_compile_pass() {
        let t = trybuild::TestCases::new();
        t.pass("tests/compile_pass/*.rs");
        t.compile_fail("tests/compile_fail/*.rs");
    }
}
