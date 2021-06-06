//! Print sizes of structs and enums
//!
//! Run with `cargo test sizes -- --nocapture`
//! to get the printed output.

use std::collections::BTreeMap;

use stencila_schema::*;

macro_rules! sizeof {
    ($type:ty) => {{
        let size = std::mem::size_of::<$type>();
        println!("{}: {}", stringify!($type), size);
        size
    }};
}

#[test]
fn sizes() {
    println!("");
    println!("----------------------------------------------");
    sizeof!(Node);
    println!("----------------------------------------------");
    sizeof!(ArrayValidator);
    sizeof!(Article);
    sizeof!(AudioObject);
    sizeof!(BooleanValidator);
    sizeof!(Brand);
    sizeof!(CitationIntentEnumeration);
    sizeof!(Cite);
    sizeof!(CiteGroup);
    sizeof!(Claim);
    sizeof!(Code);
    sizeof!(CodeBlock);
    sizeof!(CodeChunk);
    sizeof!(CodeError);
    sizeof!(CodeExpression);
    sizeof!(CodeFragment);
    sizeof!(Collection);
    sizeof!(Comment);
    sizeof!(ConstantValidator);
    sizeof!(ContactPoint);
    sizeof!(CreativeWork);
    sizeof!(Datatable);
    sizeof!(DatatableColumn);
    sizeof!(Date);
    sizeof!(DefinedTerm);
    sizeof!(Delete);
    sizeof!(Emphasis);
    sizeof!(EnumValidator);
    sizeof!(Enumeration);
    sizeof!(Figure);
    sizeof!(Function);
    sizeof!(Grant);
    sizeof!(Heading);
    sizeof!(ImageObject);
    sizeof!(Include);
    sizeof!(IntegerValidator);
    sizeof!(Link);
    sizeof!(List);
    sizeof!(ListItem);
    sizeof!(Mark);
    sizeof!(Math);
    sizeof!(MathBlock);
    sizeof!(MathFragment);
    sizeof!(MediaObject);
    sizeof!(MonetaryGrant);
    sizeof!(NontextualAnnotation);
    sizeof!(Note);
    sizeof!(NumberValidator);
    sizeof!(Organization);
    sizeof!(Paragraph);
    sizeof!(Parameter);
    sizeof!(Periodical);
    sizeof!(Person);
    sizeof!(PostalAddress);
    sizeof!(Product);
    sizeof!(PropertyValue);
    sizeof!(PublicationIssue);
    sizeof!(PublicationVolume);
    sizeof!(Quote);
    sizeof!(QuoteBlock);
    sizeof!(Review);
    sizeof!(SoftwareApplication);
    sizeof!(SoftwareEnvironment);
    sizeof!(SoftwareSession);
    sizeof!(SoftwareSourceCode);
    sizeof!(StringValidator);
    sizeof!(Strong);
    sizeof!(Subscript);
    sizeof!(Superscript);
    sizeof!(Table);
    sizeof!(TableCell);
    sizeof!(TableRow);
    sizeof!(ThematicBreak);
    sizeof!(Thing);
    sizeof!(TupleValidator);
    sizeof!(Validator);
    sizeof!(Variable);
    sizeof!(VideoObject);
    sizeof!(VolumeMount);
    println!("----------------------------------------------");

    println!("");
    println!("----------------------------------------------");
    sizeof!(InlineContent);
    println!("----------------------------------------------");
    sizeof!(AudioObjectSimple);
    sizeof!(Cite);
    sizeof!(CiteGroup);
    sizeof!(CodeExpression);
    sizeof!(CodeFragment);
    sizeof!(Delete);
    sizeof!(Emphasis);
    sizeof!(ImageObjectSimple);
    sizeof!(Link);
    sizeof!(MathFragment);
    sizeof!(NontextualAnnotation);
    sizeof!(Note);
    sizeof!(Quote);
    sizeof!(Strong);
    sizeof!(Subscript);
    sizeof!(Superscript);
    sizeof!(VideoObjectSimple);
    sizeof!(bool);
    sizeof!(i64);
    sizeof!(f64);
    sizeof!(Vec<Primitive>);
    sizeof!(BTreeMap<String, Primitive>);
    sizeof!(String);
    println!("----------------------------------------------");

    println!("");
    println!("----------------------------------------------");
    sizeof!(BlockContent);
    println!("----------------------------------------------");
    sizeof!(ClaimSimple);
    sizeof!(CodeBlock);
    sizeof!(CodeChunk);
    sizeof!(CollectionSimple);
    sizeof!(FigureSimple);
    sizeof!(Heading);
    sizeof!(List);
    sizeof!(MathBlock);
    sizeof!(Paragraph);
    sizeof!(QuoteBlock);
    sizeof!(TableSimple);
    sizeof!(ThematicBreak);
    println!("----------------------------------------------");

    println!("");
    println!("----------------------------------------------");
    sizeof!(CreativeWorkTypes);
    println!("----------------------------------------------");
    sizeof!(CreativeWork);
    sizeof!(Article);
    sizeof!(AudioObject);
    sizeof!(Claim);
    sizeof!(Collection);
    sizeof!(Comment);
    sizeof!(Datatable);
    sizeof!(Figure);
    sizeof!(ImageObject);
    sizeof!(MediaObject);
    sizeof!(Periodical);
    sizeof!(PublicationIssue);
    sizeof!(PublicationVolume);
    sizeof!(Review);
    sizeof!(SoftwareApplication);
    sizeof!(SoftwareSourceCode);
    sizeof!(Table);
    sizeof!(VideoObject);
    println!("----------------------------------------------");
}
