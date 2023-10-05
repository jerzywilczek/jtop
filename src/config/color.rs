use tui::style::Color;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SerdeColor(pub Color);

impl std::ops::Deref for SerdeColor {
    type Target = Color;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl serde::Serialize for SerdeColor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let Color::Rgb(r, g, b) = self.0 else {
            return Err(serde::ser::Error::custom(
                "only rgb colors are serializable",
            ));
        };

        serializer.serialize_str(&format!("#{:02x}{:02x}{:02x}", r, g, b))
    }
}

struct RgbColorVisitor;

impl<'de> serde::de::Visitor<'de> for RgbColorVisitor {
    type Value = SerdeColor;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string containing a hex color value looking like this: #rrggbb or like this: #RRGGBB")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if !v.is_ascii() {
            return Err(E::custom(
                "a hex color has to only contain ascii characters",
            ));
        }

        let v = v.as_bytes();

        if v.len() != 7 {
            return Err(E::custom("a hex color has to be 7 characters long"));
        }

        if v[0] != b'#' {
            return Err(E::custom("a hex color has to start with '#'"));
        }

        let v = &v[1..];

        if !v.iter().all(|c| c.is_ascii_hexdigit()) {
            return Err(E::custom(
                "the value part of a hex color has to only contain hexadecimal digits",
            ));
        }

        fn color<E: serde::de::Error>(v: [u8; 2]) -> Result<u8, E> {
            u8::from_str_radix(
                std::str::from_utf8(&[v[0].to_ascii_lowercase(), v[1].to_ascii_lowercase()])
                    .map_err(|e| E::custom(format!("unexpected error occurred: \"{}\"", e)))?,
                16,
            )
            .map_err(|e| E::custom(format!("unexpected error occurred: \"{}\"", e)))
        }

        let r = color([v[0], v[1]])?;
        let g = color([v[2], v[3]])?;
        let b = color([v[4], v[5]])?;

        Ok(SerdeColor(Color::Rgb(r, g, b)))
    }
}

impl<'de> serde::Deserialize<'de> for SerdeColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(RgbColorVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
    struct Wrapper {
        color: SerdeColor,
    }

    #[test]
    fn color_de() {
        assert_eq!(
            Wrapper {
                color: SerdeColor(tui::style::Color::Rgb(0xaa, 0xbb, 0xcc))
            },
            toml::from_str("color = \"#AABBCC\"").unwrap()
        );
        assert_eq!(
            Wrapper {
                color: SerdeColor(tui::style::Color::Rgb(0xdd, 0xee, 0xff))
            },
            toml::from_str("color = \"#ddeeff\"").unwrap()
        );
    }

    #[test]
    fn both_ways() {
        let val = Wrapper {
            color: SerdeColor(tui::style::Color::Rgb(12, 240, 144)),
        };
        assert_eq!(
            val,
            toml::from_str(&toml::to_string(&val).unwrap()).unwrap()
        )
    }
}
