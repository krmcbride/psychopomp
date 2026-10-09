//! The font database. CommitMono is compiled in, so every machine shapes the
//! same text with the same weights; installed fonts only supply glyphs it lacks
//! (chess pieces, CJK, emoji). Prose asks for Helvetica Neue and Didot, which
//! macOS installs; where they are missing, bundled Archivo and Bodoni Moda
//! answer to their names. See the licenses in `assets/fonts/`.
use cosmic_text::{Attrs, Family, FontSystem, Stretch, Style, Weight, fontdb};
use psychopomp::face::Face;

/// The bundled monospace family used for code and labels.
pub(crate) const MONO: Family<'static> = Family::Name("CommitMono");
/// The installed sans family used for prose and headers.
pub(crate) const SANS: Family<'static> = Family::Name("Helvetica Neue");
/// The installed display serif.
const SERIF: Family<'static> = Family::Name("Didot");

/// Attributes that select `face`.
pub(crate) fn attrs(face: Face) -> Attrs<'static> {
    let attrs = Attrs::new();
    match face {
        Face::Mono => attrs.family(MONO),
        Face::Sans => attrs.family(SANS),
        Face::SansBold => attrs.family(SANS).weight(Weight::BOLD),
        Face::Serif => attrs.family(SERIF),
        Face::SerifItalic => attrs.family(SERIF).style(Style::Italic),
        Face::Light => attrs.family(SANS).weight(Weight::LIGHT),
        Face::Shout => attrs
            .family(SANS)
            .stretch(Stretch::Condensed)
            .weight(Weight::BLACK),
    }
}

const COMMIT_MONO: [&[u8]; 4] = [
    include_bytes!("../../../../assets/fonts/CommitMono-400-Regular.otf"),
    include_bytes!("../../../../assets/fonts/CommitMono-400-Italic.otf"),
    include_bytes!("../../../../assets/fonts/CommitMono-700-Regular.otf"),
    include_bytes!("../../../../assets/fonts/CommitMono-700-Italic.otf"),
];

/// Stand-ins for the installed families, one face for each `Face` style.
const STAND_INS: [(&str, &[&[u8]]); 2] = [
    (
        "Helvetica Neue",
        &[
            include_bytes!("../../../../assets/fonts/Archivo-Regular.ttf"),
            include_bytes!("../../../../assets/fonts/Archivo-Bold.ttf"),
            include_bytes!("../../../../assets/fonts/Archivo-Light.ttf"),
            include_bytes!("../../../../assets/fonts/ArchivoCondensed-Black.ttf"),
        ],
    ),
    (
        "Didot",
        &[
            include_bytes!("../../../../assets/fonts/BodoniModa-Regular.ttf"),
            include_bytes!("../../../../assets/fonts/BodoniModa-Italic.ttf"),
        ],
    ),
];

/// A font system whose CommitMono faces are exactly the bundled ones, and
/// whose prose families exist on every machine.
pub(crate) fn font_system() -> FontSystem {
    let mut db = fontdb::Database::new();
    db.load_system_fonts();
    // An installed CommitMono (any version or variant) must never win a match.
    let installed = db
        .faces()
        .filter(|face| {
            face.families.iter().any(|(name, _)| {
                name.replace(' ', "")
                    .to_ascii_lowercase()
                    .starts_with("commitmono")
            })
        })
        .map(|face| face.id)
        .collect::<Vec<_>>();
    for id in installed {
        db.remove_face(id);
    }
    for font in COMMIT_MONO {
        db.load_font_data(font.to_vec());
    }
    db.set_monospace_family("CommitMono");
    for (family, fonts) in STAND_INS {
        add_stand_in(&mut db, family, fonts);
    }
    FontSystem::new_with_locale_and_db("en-US".to_owned(), db)
}

/// Register `fonts` under `family` unless it is installed. Without this a
/// missing family falls back to CommitMono, and a shout renders as code.
fn add_stand_in(db: &mut fontdb::Database, family: &str, fonts: &[&[u8]]) {
    let installed = db.faces().any(|face| {
        face.families
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case(family))
    });
    if installed {
        return;
    }
    for font in fonts {
        let source = fontdb::Source::Binary(std::sync::Arc::new(font.to_vec()));
        for id in db.load_font_source(source) {
            let mut face = db.face(id).expect("a just-loaded face").clone();
            db.remove_face(id);
            face.families = vec![(family.to_owned(), fontdb::Language::English_UnitedStates)];
            db.push_face_info(face);
        }
    }
}

#[cfg(test)]
mod tests {
    use cosmic_text::fontdb::{Query, Source, Stretch, Style, Weight};

    #[test]
    fn every_commit_mono_style_resolves_to_a_bundled_face() {
        let fonts = super::font_system();
        let db = fonts.db();
        let bundled = db
            .faces()
            .filter(|face| face.families.iter().any(|(name, _)| name == "CommitMono"))
            .collect::<Vec<_>>();
        assert_eq!(bundled.len(), 4, "exactly the four bundled faces");
        assert!(
            bundled
                .iter()
                .all(|face| matches!(face.source, Source::Binary(_)))
        );
        for (weight, style) in [
            (Weight::NORMAL, Style::Normal),
            (Weight::SEMIBOLD, Style::Normal),
            (Weight::BOLD, Style::Italic),
        ] {
            let id = db
                .query(&Query {
                    families: &[super::MONO],
                    weight,
                    stretch: Stretch::Normal,
                    style,
                })
                .expect("a CommitMono match");
            let face = db.face(id).unwrap();
            assert!(matches!(face.source, Source::Binary(_)));
            assert_eq!(face.style, style);
        }
    }

    #[test]
    fn every_prose_face_resolves_to_its_family_on_every_machine() {
        let fonts = super::font_system();
        let db = fonts.db();
        for (family, weight, stretch, style) in [
            (super::SANS, Weight::NORMAL, Stretch::Normal, Style::Normal),
            (super::SANS, Weight::BOLD, Stretch::Normal, Style::Normal),
            (super::SANS, Weight::LIGHT, Stretch::Normal, Style::Normal),
            (
                super::SANS,
                Weight::BLACK,
                Stretch::Condensed,
                Style::Normal,
            ),
            (super::SERIF, Weight::NORMAL, Stretch::Normal, Style::Normal),
            (super::SERIF, Weight::NORMAL, Stretch::Normal, Style::Italic),
        ] {
            let id = db
                .query(&Query {
                    families: &[family],
                    weight,
                    stretch,
                    style,
                })
                .unwrap_or_else(|| panic!("{family:?} {weight:?} {stretch:?} {style:?}"));
            let face = db.face(id).unwrap();
            let cosmic_text::Family::Name(name) = family else {
                unreachable!("prose families are named")
            };
            assert!(face.families.iter().any(|(n, _)| n == name));
            // Installed families are the system's business; stand-ins must be exact.
            if matches!(face.source, Source::Binary(_)) {
                assert_eq!(
                    (face.weight, face.stretch, face.style),
                    (weight, stretch, style)
                );
            }
        }
    }

    #[test]
    fn a_stand_in_never_shadows_an_installed_family() {
        let mut db = cosmic_text::fontdb::Database::new();
        super::add_stand_in(&mut db, "Archivo", &[super::STAND_INS[0].1[0]]);
        assert_eq!(db.len(), 1, "registered while Archivo is absent");
        super::add_stand_in(&mut db, "Archivo", super::STAND_INS[0].1);
        assert_eq!(db.len(), 1, "skipped once a face answers to the name");
    }
}
