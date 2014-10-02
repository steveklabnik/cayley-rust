#![feature(macro_rules)]

extern crate cayley;

use cayley::path::Vertex as V;
use cayley::path::Morphism as M;
use cayley::selector::{AnyNode, Node, Nodes};
use cayley::selector::{AnyTag, Tag, Tags};
use cayley::selector::{AnyPredicate, Predicate, Predicates};
use cayley::selector::FromQuery as Query;
use cayley::path::Path as _foo; // required to use .compile() method and other Path methods
use cayley::path::Query as _bar; // required to use Query methods

#[test]
fn main() {

    macro_rules! path_eq(
        ($src:expr, $res:expr) => (
            assert_eq!($src.compile(), Some($res.to_string()));
        );
    )

    macro_rules! path_fail(
        ($src:expr, $msg:ident) => (
            match $src.compile() {
                Some(_) => fail!($msg),
                None => ()
            };
        );
    )

    // Examples from: https://github.com/google/cayley/blob/master/docs/GremlinAPI.md

    // == Vertex ==

    // can be compiled, but not executed
    path_eq!(V::start(AnyNode), "g.V()");

    // can be compiled, but not executed
    path_eq!(V::start(Node("foo")), "g.V(\"foo\")");

    // can be compiled, but not executed
    path_eq!(V::start(Nodes(vec!("foo", "bar"))), "g.V(\"foo\", \"bar\")");

    path_eq!(V::start(AnyNode).All(), "g.V().all()");

    path_eq!(V::start(Node("foo")).All(), "g.V(\"foo\").all()");

    path_eq!(V::start(Nodes(vec!("foo", "bar"))).All(),
             "g.V(\"foo\", \"bar\").all()");

    path_eq!(V::start(Nodes(vec!("foo", "bar"))).All(),
             "g.V(\"foo\", \"bar\").all()");

    // == Morphism ==

    path_eq!(M::start().Out(Predicate("foo"), AnyTag)
                       .Out(Predicate("bar"), AnyTag),
             "g.M().Out(\"foo\").Out(\"bar\")");

    path_eq!(M::start().Out(Predicate("foo"), Tags(vec!("tag1", "tag2")))
                       .Out(Predicate("bar"), Tag("tag0")),
             "g.M().Out(\"foo\", [\"tag1\", \"tag2\"]).Out(\"bar\", \"tag0\")");

    // == Emit ==

    /* TODO: */

    // == Basic Traversals ==

    // path.Out

    path_eq!(V::start(Node("C")).Out(Predicate("follows"), AnyTag),
             "g.V(\"C\").Out(\"follows\")");

    path_eq!(V::start(Node("A")).Out(Predicate("follows"), AnyTag)
                                     .Out(Predicate("follows"), AnyTag),
             "g.V(\"A\").Out(\"follows\").Out(\"follows\")");

    path_eq!(V::start(Node("D")).Out(AnyPredicate, AnyTag),
             "g.V(\"D\").Out()");

    path_eq!(V::start(Node("D")).Out(Predicates(vec!("follows", "status")), AnyTag),
             "g.V(\"D\").Out([\"follows\", \"status\")]");

    path_eq!(V::start(Node("D")).Out(Query(V::start(Node("status"))), Tag("pred")),
             "g.V(\"D\").Out(g.V(\"status\"), \"pred\")");

    // path.In

    path_eq!(V::start(Node("cool_person")).In(Predicate("status"), AnyTag),
             "g.V(\"cool_person\").In(\"status\")");

    path_eq!(V::start(Node("B")).In(Predicate("follows"), AnyTag),
             "g.V(\"B\").In(\"follows\")");

    path_eq!(V::start(Node("E")).Out(Predicate("follows"), AnyTag)
                                .In(Predicate("follows"), AnyTag),
             "g.V(\"B\").In(\"follows\").Out(\"follows\")");

    path_eq!(V::start(Node("E")).Out(Predicate("follows"), AnyTag)
                                .In(Predicate("follows"), AnyTag),
             "g.V(\"B\").In(\"follows\").Out(\"follows\")");

    /* TODO: test with tags names & arrays */

    // path.Both

    path_eq!(V::start(Node("F")).Both(Predicate("follows"), AnyTag),
             "g.V(\"F\").Both(\"follows\")");

    // path.Is

    path_eq!(V::start(AnyNode).Out(Predicate("follows"), AnyTag).Is(Node("B")),
             "g.V().Out(\"follows\").Is(\"B\")");

    path_eq!(V::start(AnyNode).Out(Predicate("follows"), AnyTag).Is(Nodes(vec!("B", "C"))),
             "g.V().Out(\"follows\").Is(\"B\", \"C\")");

    // path.Has

    path_eq!(V::start(AnyNode).Has(Predicate("follows"), Node("B")),
             "g.V().Has(\"follows\", \"B\")");

    // == Tagging ==

    // path.Tag / path.As

    path_eq!(V::start(AnyNode).As(Tag("start")).Out(Predicate("status"), AnyTag),
             "g.V().As(\"start\").Out(\"status\")");

    path_eq!(V::start(AnyNode).Tag(Tags(vec!("foo", "bar"))).Out(Predicate("status"), AnyTag),
             "g.V().Tag(\"foo\", \"bar\").Out(\"status\")");

    // path.Back

    path_eq!(V::start(AnyNode).As(Tag("start")).Out(Predicate("status"), AnyTag)
                              .Back(Tag("start").In(Predicate("follows"), AnyTag)),
             "g.V().As(\"start\").Out(\"status\").Back(\"start\").In(\"follows\")");

    // path.Save

    path_eq!(V::start(Nodes(vec!("D", "B"))).Save(Predicate("follows"), Tag("target")),
             "g.V(\"D\", \"B\").Save(\"follows\", \"target\")");

    /* TODO:
    path_fail!(V::start(Nodes(vec!("D", "B"))).Save(AnyPredicate, Tag("target")),
               "should fail to compile path.Save w/AnyPredicate");
    path_fail!(V::start(Nodes(vec!("D", "B"))).Save(Predicates(vec!("foo", "bar")), Tag("target")),
               "should fail to compile path.Save w/Predicates");
    path_fail!(V::start(Nodes(vec!("D", "B"))).Save(Predicate("follows"), AnyTag),
               "should fail to compile path.Save w/AnyTag");
    path_fail!(V::start(Nodes(vec!("D", "B"))).Save(Predicate("follows"), Tags(vec!("foo", "bar"))),
               "should fail to compile path.Save w/AnyTag"); */

    // == Joining ==

    // path.Intersect / path.And

    let cFollows = V::start(Node("C")).Out(Predicate("follows"), AnyTag);
    let dFollows = V::start(Node("D")).Out(Predicate("follows"), AnyTag);

    path_eq!(cFollows.clone().Intersect(Query(dFollows)),
             "g.V(\"C\").Out(\"follows\").And(g.V(\"D\").Out(\"follows\"))");
    path_eq!(cFollows.clone().And(Query(dFollows)),
             "g.V(\"C\").Out(\"follows\").And(g.V(\"D\").Out(\"follows\"))");

    // path.Union / path.Or

    let cFollows = V::start(Node("C")).Out(Predicate("follows"), AnyTag);
    let dFollows = V::start(Node("D")).Out(Predicate("follows"), AnyTag);

    path_eq!(cFollows.clone().Union(Query(dFollows)),
             "g.V(\"C\").Out(\"follows\").Or(g.V(\"D\").Out(\"follows\"))");
    path_eq!(cFollows.clone().Or(Query(dFollows)),
             "g.V(\"C\").Out(\"follows\").Or(g.V(\"D\").Out(\"follows\"))");

    // == Morphisms ==

    // path.Follow

    let friendOfFriend = M::start("friendOfFriend").Out(Predicate("follows"), AnyTag)
                                                   .Out(Predicate("follows"), AnyTag);
    path_eq!(friendOfFriend, "friendOfFriend = g.M().Out(\"follows\").Out(\"follows\")");

    path_eq!(V::start(Node("C")).Follow(friendOfFriend).Has(Predicate("status"), Tag("cool_person")),
             "g.V(\"C\").Follow(friendOfFriend).Has(\"status\", \"cool_person\")");

    // path.FollowR

    path_eq!(V::start(AnyNode).Has(Predicate("status"), Tag("cool_person")).FollowR(friendOfFriend),
             "g.V(\"C\").Has(\"status\", \"cool_person\").FollowR(friendOfFriend)");

    // == Query finals ==

    path_eq!(V::start(AnyNode).Out(Predicate("follows"), AnyTag).All(),
             "g.V().Out(\"follows\").All()");

    path_eq!(V::start(Node("foo")).Out(Predicate("follows"), AnyTag).GetLimit(5),
             "g.V(\"foo\").Out(\"follows\").GetLimit(5)");

    path_eq!(V::start(Node("bar")).In(Predicate("follows"), AnyTag).ToArray(),
             "g.V(\"bar\").In(\"follows\").toArray()");

    path_eq!(V::start(AnyNode).Out(Predicate("follows"), AnyTag).ToValue(),
             "g.V().Out(\"follows\").ToValue()");

    path_eq!(V::start(Node("foo")).Out(Predicate("follows"), AnyTag).TagValue(),
             "g.V(\"foo\").Out(\"follows\").TagValue()");

    /* TODO: query.ForEach(callback), query.ForEach(limit, callback) */

}
