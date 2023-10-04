#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RgbColor(pub u8, pub u8, pub u8);

impl serde::Serialize for RgbColor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("#{:02x}{:02x}{:02x}", self.0, self.1, self.2))
    }
}

struct RgbColorVisitor;

impl<'de> serde::de::Visitor<'de> for RgbColorVisitor {
    type Value = RgbColor;

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

        Ok(RgbColor(r, g, b))
    }
}

impl<'de> serde::Deserialize<'de> for RgbColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(RgbColorVisitor)
    }
}

impl From<RgbColor> for tui::style::Color {
    fn from(value: RgbColor) -> Self {
        tui::style::Color::Rgb(value.0, value.1, value.2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
    struct Wrapper {
        color: RgbColor,
    }

    #[test]
    fn color_de() {
        assert_eq!(
            Wrapper {
                color: RgbColor(0xaa, 0xbb, 0xcc)
            },
            toml::from_str("color = \"#AABBCC\"").unwrap()
        );
        assert_eq!(
            Wrapper {
                color: RgbColor(0xdd, 0xee, 0xff)
            },
            toml::from_str("color = \"#ddeeff\"").unwrap()
        );
    }

    #[test]
    fn both_ways() {
        let val = Wrapper {
            color: RgbColor(12, 240, 144),
        };
        assert_eq!(
            val,
            toml::from_str(&toml::to_string(&val).unwrap()).unwrap()
        )
    }
}
