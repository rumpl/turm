#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graphic_rendition_from_u8() {
        use crate::color::Color;
        use crate::ansi::GraphicRendition;

        // Test basic formatting codes
        assert!(matches!(GraphicRendition::from(0), GraphicRendition::Reset));
        assert!(matches!(GraphicRendition::from(1), GraphicRendition::Bold));
        assert!(matches!(GraphicRendition::from(2), GraphicRendition::Dim));
        assert!(matches!(GraphicRendition::from(3), GraphicRendition::Italic));
        assert!(matches!(GraphicRendition::from(4), GraphicRendition::Underline));
        assert!(matches!(GraphicRendition::from(5), GraphicRendition::Blink));
        assert!(matches!(GraphicRendition::from(7), GraphicRendition::Reverse));
        assert!(matches!(GraphicRendition::from(8), GraphicRendition::Hidden));
        assert!(matches!(GraphicRendition::from(9), GraphicRendition::StrikeThrough));

        // Test standard foreground colors
        assert!(matches!(GraphicRendition::from(30), GraphicRendition::ForegroundColor(_)));
        assert!(matches!(GraphicRendition::from(31), GraphicRendition::ForegroundColor(_)));
        
        // Test bright foreground colors
        assert!(matches!(GraphicRendition::from(90), GraphicRendition::ForegroundColor(_)));
        assert!(matches!(GraphicRendition::from(91), GraphicRendition::ForegroundColor(_)));
        
        // Test standard background colors
        assert!(matches!(GraphicRendition::from(40), GraphicRendition::BackgroundColor(_)));
        assert!(matches!(GraphicRendition::from(41), GraphicRendition::BackgroundColor(_)));
        
        // Test bright background colors
        assert!(matches!(GraphicRendition::from(100), GraphicRendition::BackgroundColor(_)));
        assert!(matches!(GraphicRendition::from(101), GraphicRendition::BackgroundColor(_)));
    }
}