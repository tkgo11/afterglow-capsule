use ag_schema::*;
use serde_json::json;

#[test]
fn every_field_type_round_trips_and_checks_its_value() {
    for (kind, value) in [
        (FieldType::ShortText, json!("Name")),
        (FieldType::LongText, json!("Text")),
        (FieldType::Number, json!(12.5)),
        (FieldType::Date, json!("2030-01-01")),
        (
            FieldType::DateRange,
            json!({"start":"2030-01-01", "end":"2030-02-01"}),
        ),
        (FieldType::SingleSelect, json!("a")),
        (FieldType::MultiSelect, json!(["a", "b"])),
        (FieldType::Boolean, json!(true)),
        (FieldType::Image, json!(StableId::random())),
        (FieldType::TagList, json!(["one", "two"])),
        (FieldType::Url, json!("https://example.com/page")),
        (FieldType::HiddenId, json!(StableId::random())),
    ] {
        let choices = if matches!(kind, FieldType::SingleSelect | FieldType::MultiSelect) {
            vec!["a".into(), "b".into()]
        } else {
            vec![]
        };
        let schema = CustomSchema {
            fields: vec![FieldDefinition {
                id: "field".into(),
                label: "Generic field".into(),
                field_type: kind,
                visibility: Visibility::PrivateEncrypted,
                required: true,
                choices,
            }],
        };
        let values = FieldValues::from([("field".into(), value)]);
        schema.validate_values(&values).unwrap();
        assert!(schema.validate_values(&FieldValues::new()).is_err());
        let doc = Document::new("afterglow-schema", schema).unwrap();
        assert_eq!(
            doc,
            Document::<CustomSchema>::from_json(
                &doc.to_json("afterglow-schema").unwrap(),
                "afterglow-schema"
            )
            .unwrap()
        );
    }
}

#[test]
fn schema_rejects_duplicates_unknown_values_wrong_types_and_reversed_dates() {
    let field = FieldDefinition {
        id: "field".into(),
        label: "Field".into(),
        field_type: FieldType::Boolean,
        visibility: Visibility::PrivateEncrypted,
        required: true,
        choices: vec![],
    };
    let mut schema = CustomSchema {
        fields: vec![field.clone()],
    };
    assert!(
        schema
            .validate_values(&FieldValues::from([("field".into(), json!("true"))]))
            .is_err()
    );
    assert!(
        schema
            .validate_values(&FieldValues::from([("script".into(), json!(true))]))
            .is_err()
    );
    schema.fields.push(field);
    assert!(schema.validate().is_err());
    schema.fields.truncate(1);
    schema.fields[0].field_type = FieldType::DateRange;
    assert!(
        schema
            .validate_values(&FieldValues::from([(
                "field".into(),
                json!({"start":"2030-02-01", "end":"2030-01-01"})
            )]))
            .is_err()
    );
}

#[test]
fn declarative_content_round_trips_and_never_accepts_executable_nodes_or_link_schemes() {
    let id = StableId::random();
    let text = vec![
        Inline::Text {
            text: "Literal <script>text</script>".into(),
            bold: true,
            italic: false,
            underline: false,
            emphasis: true,
            link: None,
        },
        Inline::SoftBreak,
    ];
    let date = "2030-01-01".parse().unwrap();
    let blocks = vec![
        ContentBlock::Heading {
            level: 1,
            content: text.clone(),
        },
        ContentBlock::Paragraph {
            content: text.clone(),
        },
        ContentBlock::Quote {
            content: text.clone(),
        },
        ContentBlock::Signature {
            content: text.clone(),
        },
        ContentBlock::Divider,
        ContentBlock::Image {
            object_id: id,
            alt: "Image".into(),
        },
        ContentBlock::ImagePair {
            object_ids: [id, id],
            alt: ["Left".into(), "Right".into()],
        },
        ContentBlock::Gallery {
            object_ids: vec![id],
            alt: vec!["Image".into()],
        },
        ContentBlock::Caption {
            content: text.clone(),
        },
        ContentBlock::Timeline {
            items: vec![TimelineItem {
                date,
                content: text.clone(),
            }],
        },
        ContentBlock::Callout {
            content: text.clone(),
        },
        ContentBlock::DateStamp { date },
        ContentBlock::Credits {
            content: text.clone(),
        },
        ContentBlock::Spacer,
        ContentBlock::Link {
            label: "External".into(),
            url: "https://example.com".into(),
        },
        ContentBlock::Audio {
            object_id: id,
            caption: text.clone(),
        },
    ];
    for block in &blocks {
        block.validate().unwrap();
    }
    let encoded = serde_json::to_vec(&blocks).unwrap();
    assert_eq!(
        blocks,
        serde_json::from_slice::<Vec<ContentBlock>>(&encoded).unwrap()
    );
    for kind in ["script", "html", "plugin", "iframe"] {
        assert!(
            serde_json::from_value::<ContentBlock>(json!({"kind":kind,"code":"alert(1)"})).is_err()
        );
    }
    for url in [
        "javascript:alert(1)",
        "data:text/html,script",
        "file:///secret",
        "https://user:secret@example.com",
    ] {
        assert!(
            ContentBlock::Link {
                label: "Link".into(),
                url: url.into()
            }
            .validate()
            .is_err()
        );
    }
    assert!(
        ContentBlock::Heading {
            level: 0,
            content: text
        }
        .validate()
        .is_err()
    );
    assert!(
        ContentBlock::Gallery {
            object_ids: vec![id],
            alt: vec![]
        }
        .validate()
        .is_err()
    );
}
