use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum MaybeRef<'sch, T> {
    #[serde(borrow)]
    Ref(Ref<'sch>),
    Owned(T),
}

#[derive(Debug, Clone, Deserialize)]
pub struct Ref<'sch> {
    #[serde(rename = "$ref")]
    pub target: &'sch str,
}

#[cfg(test)]
#[test]
fn test_maybe_ref() {
    #[derive(Deserialize)]
    struct Foo {
        a: u32,
    }

    let ref_: MaybeRef<'static, Foo> = serde_json::from_str(
        r##"
    {
        "$ref": "foo"
    }
    "##,
    )
    .unwrap();
    assert!(matches!(ref_, MaybeRef::Ref(Ref { target: "foo" })));

    let owned: MaybeRef<'static, Foo> = serde_json::from_str(
        r##"
    {
        "a": 3
    }
    "##,
    )
    .unwrap();
    assert!(matches!(owned, MaybeRef::Owned(Foo { a: 3 })));
}
