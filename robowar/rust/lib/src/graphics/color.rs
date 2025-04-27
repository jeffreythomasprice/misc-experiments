use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct RGBAu8 {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct RGBAf32 {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

impl RGBAu8 {
    // https://www.w3.org/TR/css-color-4/#named-colors
    pub const ALICEBLUE: Self = Self::new(240, 248, 255, 255);
    pub const ANTIQUEWHITE: Self = Self::new(250, 235, 215, 255);
    pub const AQUA: Self = Self::new(0, 255, 255, 255);
    pub const AQUAMARINE: Self = Self::new(127, 255, 212, 255);
    pub const AZURE: Self = Self::new(240, 255, 255, 255);
    pub const BEIGE: Self = Self::new(245, 245, 220, 255);
    pub const BISQUE: Self = Self::new(255, 228, 196, 255);
    pub const BLACK: Self = Self::new(0, 0, 0, 255);
    pub const BLANCHEDALMOND: Self = Self::new(255, 235, 205, 255);
    pub const BLUE: Self = Self::new(0, 0, 255, 255);
    pub const BLUEVIOLET: Self = Self::new(138, 43, 226, 255);
    pub const BROWN: Self = Self::new(165, 42, 42, 255);
    pub const BURLYWOOD: Self = Self::new(222, 184, 135, 255);
    pub const CADETBLUE: Self = Self::new(95, 158, 160, 255);
    pub const CHARTREUSE: Self = Self::new(127, 255, 0, 255);
    pub const CHOCOLATE: Self = Self::new(210, 105, 30, 255);
    pub const CORAL: Self = Self::new(255, 127, 80, 255);
    pub const CORNFLOWERBLUE: Self = Self::new(100, 149, 237, 255);
    pub const CORNSILK: Self = Self::new(255, 248, 220, 255);
    pub const CRIMSON: Self = Self::new(220, 20, 60, 255);
    pub const CYAN: Self = Self::new(0, 255, 255, 255);
    pub const DARKBLUE: Self = Self::new(0, 0, 139, 255);
    pub const DARKCYAN: Self = Self::new(0, 139, 139, 255);
    pub const DARKGOLDENROD: Self = Self::new(184, 134, 11, 255);
    pub const DARKGRAY: Self = Self::new(169, 169, 169, 255);
    pub const DARKGREEN: Self = Self::new(0, 100, 0, 255);
    pub const DARKGREY: Self = Self::new(169, 169, 169, 255);
    pub const DARKKHAKI: Self = Self::new(189, 183, 107, 255);
    pub const DARKMAGENTA: Self = Self::new(139, 0, 139, 255);
    pub const DARKOLIVEGREEN: Self = Self::new(85, 107, 47, 255);
    pub const DARKORANGE: Self = Self::new(255, 140, 0, 255);
    pub const DARKORCHID: Self = Self::new(153, 50, 204, 255);
    pub const DARKRED: Self = Self::new(139, 0, 0, 255);
    pub const DARKSALMON: Self = Self::new(233, 150, 122, 255);
    pub const DARKSEAGREEN: Self = Self::new(143, 188, 143, 255);
    pub const DARKSLATEBLUE: Self = Self::new(72, 61, 139, 255);
    pub const DARKSLATEGRAY: Self = Self::new(47, 79, 79, 255);
    pub const DARKSLATEGREY: Self = Self::new(47, 79, 79, 255);
    pub const DARKTURQUOISE: Self = Self::new(0, 206, 209, 255);
    pub const DARKVIOLET: Self = Self::new(148, 0, 211, 255);
    pub const DEEPPINK: Self = Self::new(255, 20, 147, 255);
    pub const DEEPSKYBLUE: Self = Self::new(0, 191, 255, 255);
    pub const DIMGRAY: Self = Self::new(105, 105, 105, 255);
    pub const DIMGREY: Self = Self::new(105, 105, 105, 255);
    pub const DODGERBLUE: Self = Self::new(30, 144, 255, 255);
    pub const FIREBRICK: Self = Self::new(178, 34, 34, 255);
    pub const FLORALWHITE: Self = Self::new(255, 250, 240, 255);
    pub const FORESTGREEN: Self = Self::new(34, 139, 34, 255);
    pub const FUCHSIA: Self = Self::new(255, 0, 255, 255);
    pub const GAINSBORO: Self = Self::new(220, 220, 220, 255);
    pub const GHOSTWHITE: Self = Self::new(248, 248, 255, 255);
    pub const GOLD: Self = Self::new(255, 215, 0, 255);
    pub const GOLDENROD: Self = Self::new(218, 165, 32, 255);
    pub const GRAY: Self = Self::new(128, 128, 128, 255);
    pub const GREEN: Self = Self::new(0, 128, 0, 255);
    pub const GREENYELLOW: Self = Self::new(173, 255, 47, 255);
    pub const GREY: Self = Self::new(128, 128, 128, 255);
    pub const HONEYDEW: Self = Self::new(240, 255, 240, 255);
    pub const HOTPINK: Self = Self::new(255, 105, 180, 255);
    pub const INDIANRED: Self = Self::new(205, 92, 92, 255);
    pub const INDIGO: Self = Self::new(75, 0, 130, 255);
    pub const IVORY: Self = Self::new(255, 255, 240, 255);
    pub const KHAKI: Self = Self::new(240, 230, 140, 255);
    pub const LAVENDER: Self = Self::new(230, 230, 250, 255);
    pub const LAVENDERBLUSH: Self = Self::new(255, 240, 245, 255);
    pub const LAWNGREEN: Self = Self::new(124, 252, 0, 255);
    pub const LEMONCHIFFON: Self = Self::new(255, 250, 205, 255);
    pub const LIGHTBLUE: Self = Self::new(173, 216, 230, 255);
    pub const LIGHTCORAL: Self = Self::new(240, 128, 128, 255);
    pub const LIGHTCYAN: Self = Self::new(224, 255, 255, 255);
    pub const LIGHTGOLDENRODYELLOW: Self = Self::new(250, 250, 210, 255);
    pub const LIGHTGRAY: Self = Self::new(211, 211, 211, 255);
    pub const LIGHTGREEN: Self = Self::new(144, 238, 144, 255);
    pub const LIGHTGREY: Self = Self::new(211, 211, 211, 255);
    pub const LIGHTPINK: Self = Self::new(255, 182, 193, 255);
    pub const LIGHTSALMON: Self = Self::new(255, 160, 122, 255);
    pub const LIGHTSEAGREEN: Self = Self::new(32, 178, 170, 255);
    pub const LIGHTSKYBLUE: Self = Self::new(135, 206, 250, 255);
    pub const LIGHTSLATEGRAY: Self = Self::new(119, 136, 153, 255);
    pub const LIGHTSLATEGREY: Self = Self::new(119, 136, 153, 255);
    pub const LIGHTSTEELBLUE: Self = Self::new(176, 196, 222, 255);
    pub const LIGHTYELLOW: Self = Self::new(255, 255, 224, 255);
    pub const LIME: Self = Self::new(0, 255, 0, 255);
    pub const LIMEGREEN: Self = Self::new(50, 205, 50, 255);
    pub const LINEN: Self = Self::new(250, 240, 230, 255);
    pub const MAGENTA: Self = Self::new(255, 0, 255, 255);
    pub const MAROON: Self = Self::new(128, 0, 0, 255);
    pub const MEDIUMAQUAMARINE: Self = Self::new(102, 205, 170, 255);
    pub const MEDIUMBLUE: Self = Self::new(0, 0, 205, 255);
    pub const MEDIUMORCHID: Self = Self::new(186, 85, 211, 255);
    pub const MEDIUMPURPLE: Self = Self::new(147, 112, 219, 255);
    pub const MEDIUMSEAGREEN: Self = Self::new(60, 179, 113, 255);
    pub const MEDIUMSLATEBLUE: Self = Self::new(123, 104, 238, 255);
    pub const MEDIUMSPRINGGREEN: Self = Self::new(0, 250, 154, 255);
    pub const MEDIUMTURQUOISE: Self = Self::new(72, 209, 204, 255);
    pub const MEDIUMVIOLETRED: Self = Self::new(199, 21, 133, 255);
    pub const MIDNIGHTBLUE: Self = Self::new(25, 25, 112, 255);
    pub const MINTCREAM: Self = Self::new(245, 255, 250, 255);
    pub const MISTYROSE: Self = Self::new(255, 228, 225, 255);
    pub const MOCCASIN: Self = Self::new(255, 228, 181, 255);
    pub const NAVAJOWHITE: Self = Self::new(255, 222, 173, 255);
    pub const NAVY: Self = Self::new(0, 0, 128, 255);
    pub const OLDLACE: Self = Self::new(253, 245, 230, 255);
    pub const OLIVE: Self = Self::new(128, 128, 0, 255);
    pub const OLIVEDRAB: Self = Self::new(107, 142, 35, 255);
    pub const ORANGE: Self = Self::new(255, 165, 0, 255);
    pub const ORANGERED: Self = Self::new(255, 69, 0, 255);
    pub const ORCHID: Self = Self::new(218, 112, 214, 255);
    pub const PALEGOLDENROD: Self = Self::new(238, 232, 170, 255);
    pub const PALEGREEN: Self = Self::new(152, 251, 152, 255);
    pub const PALETURQUOISE: Self = Self::new(175, 238, 238, 255);
    pub const PALEVIOLETRED: Self = Self::new(219, 112, 147, 255);
    pub const PAPAYAWHIP: Self = Self::new(255, 239, 213, 255);
    pub const PEACHPUFF: Self = Self::new(255, 218, 185, 255);
    pub const PERU: Self = Self::new(205, 133, 63, 255);
    pub const PINK: Self = Self::new(255, 192, 203, 255);
    pub const PLUM: Self = Self::new(221, 160, 221, 255);
    pub const POWDERBLUE: Self = Self::new(176, 224, 230, 255);
    pub const PURPLE: Self = Self::new(128, 0, 128, 255);
    pub const REBECCAPURPLE: Self = Self::new(102, 51, 153, 255);
    pub const RED: Self = Self::new(255, 0, 0, 255);
    pub const ROSYBROWN: Self = Self::new(188, 143, 143, 255);
    pub const ROYALBLUE: Self = Self::new(65, 105, 225, 255);
    pub const SADDLEBROWN: Self = Self::new(139, 69, 19, 255);
    pub const SALMON: Self = Self::new(250, 128, 114, 255);
    pub const SANDYBROWN: Self = Self::new(244, 164, 96, 255);
    pub const SEAGREEN: Self = Self::new(46, 139, 87, 255);
    pub const SEASHELL: Self = Self::new(255, 245, 238, 255);
    pub const SIENNA: Self = Self::new(160, 82, 45, 255);
    pub const SILVER: Self = Self::new(192, 192, 192, 255);
    pub const SKYBLUE: Self = Self::new(135, 206, 235, 255);
    pub const SLATEBLUE: Self = Self::new(106, 90, 205, 255);
    pub const SLATEGRAY: Self = Self::new(112, 128, 144, 255);
    pub const SLATEGREY: Self = Self::new(112, 128, 144, 255);
    pub const SNOW: Self = Self::new(255, 250, 250, 255);
    pub const SPRINGGREEN: Self = Self::new(0, 255, 127, 255);
    pub const STEELBLUE: Self = Self::new(70, 130, 180, 255);
    pub const TAN: Self = Self::new(210, 180, 140, 255);
    pub const TEAL: Self = Self::new(0, 128, 128, 255);
    pub const THISTLE: Self = Self::new(216, 191, 216, 255);
    pub const TOMATO: Self = Self::new(255, 99, 71, 255);
    pub const TURQUOISE: Self = Self::new(64, 224, 208, 255);
    pub const VIOLET: Self = Self::new(238, 130, 238, 255);
    pub const WHEAT: Self = Self::new(245, 222, 179, 255);
    pub const WHITE: Self = Self::new(255, 255, 255, 255);
    pub const WHITESMOKE: Self = Self::new(245, 245, 245, 255);
    pub const YELLOW: Self = Self::new(255, 255, 0, 255);
    pub const YELLOWGREEN: Self = Self::new(154, 205, 50, 255);

    pub const fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }
}

impl RGBAf32 {
    pub const ALICEBLUE: Self =
        Self::new(240.0 / 255.0, 248.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0);
    pub const ANTIQUEWHITE: Self =
        Self::new(250.0 / 255.0, 235.0 / 255.0, 215.0 / 255.0, 255.0 / 255.0);
    pub const AQUA: Self = Self::new(0.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0);
    pub const AQUAMARINE: Self =
        Self::new(127.0 / 255.0, 255.0 / 255.0, 212.0 / 255.0, 255.0 / 255.0);
    pub const AZURE: Self = Self::new(240.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0);
    pub const BEIGE: Self = Self::new(245.0 / 255.0, 245.0 / 255.0, 220.0 / 255.0, 255.0 / 255.0);
    pub const BISQUE: Self = Self::new(255.0 / 255.0, 228.0 / 255.0, 196.0 / 255.0, 255.0 / 255.0);
    pub const BLACK: Self = Self::new(0.0 / 255.0, 0.0 / 255.0, 0.0 / 255.0, 255.0 / 255.0);
    pub const BLANCHEDALMOND: Self =
        Self::new(255.0 / 255.0, 235.0 / 255.0, 205.0 / 255.0, 255.0 / 255.0);
    pub const BLUE: Self = Self::new(0.0 / 255.0, 0.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0);
    pub const BLUEVIOLET: Self =
        Self::new(138.0 / 255.0, 43.0 / 255.0, 226.0 / 255.0, 255.0 / 255.0);
    pub const BROWN: Self = Self::new(165.0 / 255.0, 42.0 / 255.0, 42.0 / 255.0, 255.0 / 255.0);
    pub const BURLYWOOD: Self =
        Self::new(222.0 / 255.0, 184.0 / 255.0, 135.0 / 255.0, 255.0 / 255.0);
    pub const CADETBLUE: Self =
        Self::new(95.0 / 255.0, 158.0 / 255.0, 160.0 / 255.0, 255.0 / 255.0);
    pub const CHARTREUSE: Self =
        Self::new(127.0 / 255.0, 255.0 / 255.0, 0.0 / 255.0, 255.0 / 255.0);
    pub const CHOCOLATE: Self =
        Self::new(210.0 / 255.0, 105.0 / 255.0, 30.0 / 255.0, 255.0 / 255.0);
    pub const CORAL: Self = Self::new(255.0 / 255.0, 127.0 / 255.0, 80.0 / 255.0, 255.0 / 255.0);
    pub const CORNFLOWERBLUE: Self =
        Self::new(100.0 / 255.0, 149.0 / 255.0, 237.0 / 255.0, 255.0 / 255.0);
    pub const CORNSILK: Self =
        Self::new(255.0 / 255.0, 248.0 / 255.0, 220.0 / 255.0, 255.0 / 255.0);
    pub const CRIMSON: Self = Self::new(220.0 / 255.0, 20.0 / 255.0, 60.0 / 255.0, 255.0 / 255.0);
    pub const CYAN: Self = Self::new(0.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0);
    pub const DARKBLUE: Self = Self::new(0.0 / 255.0, 0.0 / 255.0, 139.0 / 255.0, 255.0 / 255.0);
    pub const DARKCYAN: Self = Self::new(0.0 / 255.0, 139.0 / 255.0, 139.0 / 255.0, 255.0 / 255.0);
    pub const DARKGOLDENROD: Self =
        Self::new(184.0 / 255.0, 134.0 / 255.0, 11.0 / 255.0, 255.0 / 255.0);
    pub const DARKGRAY: Self =
        Self::new(169.0 / 255.0, 169.0 / 255.0, 169.0 / 255.0, 255.0 / 255.0);
    pub const DARKGREEN: Self = Self::new(0.0 / 255.0, 100.0 / 255.0, 0.0 / 255.0, 255.0 / 255.0);
    pub const DARKGREY: Self =
        Self::new(169.0 / 255.0, 169.0 / 255.0, 169.0 / 255.0, 255.0 / 255.0);
    pub const DARKKHAKI: Self =
        Self::new(189.0 / 255.0, 183.0 / 255.0, 107.0 / 255.0, 255.0 / 255.0);
    pub const DARKMAGENTA: Self =
        Self::new(139.0 / 255.0, 0.0 / 255.0, 139.0 / 255.0, 255.0 / 255.0);
    pub const DARKOLIVEGREEN: Self =
        Self::new(85.0 / 255.0, 107.0 / 255.0, 47.0 / 255.0, 255.0 / 255.0);
    pub const DARKORANGE: Self =
        Self::new(255.0 / 255.0, 140.0 / 255.0, 0.0 / 255.0, 255.0 / 255.0);
    pub const DARKORCHID: Self =
        Self::new(153.0 / 255.0, 50.0 / 255.0, 204.0 / 255.0, 255.0 / 255.0);
    pub const DARKRED: Self = Self::new(139.0 / 255.0, 0.0 / 255.0, 0.0 / 255.0, 255.0 / 255.0);
    pub const DARKSALMON: Self =
        Self::new(233.0 / 255.0, 150.0 / 255.0, 122.0 / 255.0, 255.0 / 255.0);
    pub const DARKSEAGREEN: Self =
        Self::new(143.0 / 255.0, 188.0 / 255.0, 143.0 / 255.0, 255.0 / 255.0);
    pub const DARKSLATEBLUE: Self =
        Self::new(72.0 / 255.0, 61.0 / 255.0, 139.0 / 255.0, 255.0 / 255.0);
    pub const DARKSLATEGRAY: Self =
        Self::new(47.0 / 255.0, 79.0 / 255.0, 79.0 / 255.0, 255.0 / 255.0);
    pub const DARKSLATEGREY: Self =
        Self::new(47.0 / 255.0, 79.0 / 255.0, 79.0 / 255.0, 255.0 / 255.0);
    pub const DARKTURQUOISE: Self =
        Self::new(0.0 / 255.0, 206.0 / 255.0, 209.0 / 255.0, 255.0 / 255.0);
    pub const DARKVIOLET: Self =
        Self::new(148.0 / 255.0, 0.0 / 255.0, 211.0 / 255.0, 255.0 / 255.0);
    pub const DEEPPINK: Self = Self::new(255.0 / 255.0, 20.0 / 255.0, 147.0 / 255.0, 255.0 / 255.0);
    pub const DEEPSKYBLUE: Self =
        Self::new(0.0 / 255.0, 191.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0);
    pub const DIMGRAY: Self = Self::new(105.0 / 255.0, 105.0 / 255.0, 105.0 / 255.0, 255.0 / 255.0);
    pub const DIMGREY: Self = Self::new(105.0 / 255.0, 105.0 / 255.0, 105.0 / 255.0, 255.0 / 255.0);
    pub const DODGERBLUE: Self =
        Self::new(30.0 / 255.0, 144.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0);
    pub const FIREBRICK: Self = Self::new(178.0 / 255.0, 34.0 / 255.0, 34.0 / 255.0, 255.0 / 255.0);
    pub const FLORALWHITE: Self =
        Self::new(255.0 / 255.0, 250.0 / 255.0, 240.0 / 255.0, 255.0 / 255.0);
    pub const FORESTGREEN: Self =
        Self::new(34.0 / 255.0, 139.0 / 255.0, 34.0 / 255.0, 255.0 / 255.0);
    pub const FUCHSIA: Self = Self::new(255.0 / 255.0, 0.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0);
    pub const GAINSBORO: Self =
        Self::new(220.0 / 255.0, 220.0 / 255.0, 220.0 / 255.0, 255.0 / 255.0);
    pub const GHOSTWHITE: Self =
        Self::new(248.0 / 255.0, 248.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0);
    pub const GOLD: Self = Self::new(255.0 / 255.0, 215.0 / 255.0, 0.0 / 255.0, 255.0 / 255.0);
    pub const GOLDENROD: Self =
        Self::new(218.0 / 255.0, 165.0 / 255.0, 32.0 / 255.0, 255.0 / 255.0);
    pub const GRAY: Self = Self::new(128.0 / 255.0, 128.0 / 255.0, 128.0 / 255.0, 255.0 / 255.0);
    pub const GREEN: Self = Self::new(0.0 / 255.0, 128.0 / 255.0, 0.0 / 255.0, 255.0 / 255.0);
    pub const GREENYELLOW: Self =
        Self::new(173.0 / 255.0, 255.0 / 255.0, 47.0 / 255.0, 255.0 / 255.0);
    pub const GREY: Self = Self::new(128.0 / 255.0, 128.0 / 255.0, 128.0 / 255.0, 255.0 / 255.0);
    pub const HONEYDEW: Self =
        Self::new(240.0 / 255.0, 255.0 / 255.0, 240.0 / 255.0, 255.0 / 255.0);
    pub const HOTPINK: Self = Self::new(255.0 / 255.0, 105.0 / 255.0, 180.0 / 255.0, 255.0 / 255.0);
    pub const INDIANRED: Self = Self::new(205.0 / 255.0, 92.0 / 255.0, 92.0 / 255.0, 255.0 / 255.0);
    pub const INDIGO: Self = Self::new(75.0 / 255.0, 0.0 / 255.0, 130.0 / 255.0, 255.0 / 255.0);
    pub const IVORY: Self = Self::new(255.0 / 255.0, 255.0 / 255.0, 240.0 / 255.0, 255.0 / 255.0);
    pub const KHAKI: Self = Self::new(240.0 / 255.0, 230.0 / 255.0, 140.0 / 255.0, 255.0 / 255.0);
    pub const LAVENDER: Self =
        Self::new(230.0 / 255.0, 230.0 / 255.0, 250.0 / 255.0, 255.0 / 255.0);
    pub const LAVENDERBLUSH: Self =
        Self::new(255.0 / 255.0, 240.0 / 255.0, 245.0 / 255.0, 255.0 / 255.0);
    pub const LAWNGREEN: Self = Self::new(124.0 / 255.0, 252.0 / 255.0, 0.0 / 255.0, 255.0 / 255.0);
    pub const LEMONCHIFFON: Self =
        Self::new(255.0 / 255.0, 250.0 / 255.0, 205.0 / 255.0, 255.0 / 255.0);
    pub const LIGHTBLUE: Self =
        Self::new(173.0 / 255.0, 216.0 / 255.0, 230.0 / 255.0, 255.0 / 255.0);
    pub const LIGHTCORAL: Self =
        Self::new(240.0 / 255.0, 128.0 / 255.0, 128.0 / 255.0, 255.0 / 255.0);
    pub const LIGHTCYAN: Self =
        Self::new(224.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0);
    pub const LIGHTGOLDENRODYELLOW: Self =
        Self::new(250.0 / 255.0, 250.0 / 255.0, 210.0 / 255.0, 255.0 / 255.0);
    pub const LIGHTGRAY: Self =
        Self::new(211.0 / 255.0, 211.0 / 255.0, 211.0 / 255.0, 255.0 / 255.0);
    pub const LIGHTGREEN: Self =
        Self::new(144.0 / 255.0, 238.0 / 255.0, 144.0 / 255.0, 255.0 / 255.0);
    pub const LIGHTGREY: Self =
        Self::new(211.0 / 255.0, 211.0 / 255.0, 211.0 / 255.0, 255.0 / 255.0);
    pub const LIGHTPINK: Self =
        Self::new(255.0 / 255.0, 182.0 / 255.0, 193.0 / 255.0, 255.0 / 255.0);
    pub const LIGHTSALMON: Self =
        Self::new(255.0 / 255.0, 160.0 / 255.0, 122.0 / 255.0, 255.0 / 255.0);
    pub const LIGHTSEAGREEN: Self =
        Self::new(32.0 / 255.0, 178.0 / 255.0, 170.0 / 255.0, 255.0 / 255.0);
    pub const LIGHTSKYBLUE: Self =
        Self::new(135.0 / 255.0, 206.0 / 255.0, 250.0 / 255.0, 255.0 / 255.0);
    pub const LIGHTSLATEGRAY: Self =
        Self::new(119.0 / 255.0, 136.0 / 255.0, 153.0 / 255.0, 255.0 / 255.0);
    pub const LIGHTSLATEGREY: Self =
        Self::new(119.0 / 255.0, 136.0 / 255.0, 153.0 / 255.0, 255.0 / 255.0);
    pub const LIGHTSTEELBLUE: Self =
        Self::new(176.0 / 255.0, 196.0 / 255.0, 222.0 / 255.0, 255.0 / 255.0);
    pub const LIGHTYELLOW: Self =
        Self::new(255.0 / 255.0, 255.0 / 255.0, 224.0 / 255.0, 255.0 / 255.0);
    pub const LIME: Self = Self::new(0.0 / 255.0, 255.0 / 255.0, 0.0 / 255.0, 255.0 / 255.0);
    pub const LIMEGREEN: Self = Self::new(50.0 / 255.0, 205.0 / 255.0, 50.0 / 255.0, 255.0 / 255.0);
    pub const LINEN: Self = Self::new(250.0 / 255.0, 240.0 / 255.0, 230.0 / 255.0, 255.0 / 255.0);
    pub const MAGENTA: Self = Self::new(255.0 / 255.0, 0.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0);
    pub const MAROON: Self = Self::new(128.0 / 255.0, 0.0 / 255.0, 0.0 / 255.0, 255.0 / 255.0);
    pub const MEDIUMAQUAMARINE: Self =
        Self::new(102.0 / 255.0, 205.0 / 255.0, 170.0 / 255.0, 255.0 / 255.0);
    pub const MEDIUMBLUE: Self = Self::new(0.0 / 255.0, 0.0 / 255.0, 205.0 / 255.0, 255.0 / 255.0);
    pub const MEDIUMORCHID: Self =
        Self::new(186.0 / 255.0, 85.0 / 255.0, 211.0 / 255.0, 255.0 / 255.0);
    pub const MEDIUMPURPLE: Self =
        Self::new(147.0 / 255.0, 112.0 / 255.0, 219.0 / 255.0, 255.0 / 255.0);
    pub const MEDIUMSEAGREEN: Self =
        Self::new(60.0 / 255.0, 179.0 / 255.0, 113.0 / 255.0, 255.0 / 255.0);
    pub const MEDIUMSLATEBLUE: Self =
        Self::new(123.0 / 255.0, 104.0 / 255.0, 238.0 / 255.0, 255.0 / 255.0);
    pub const MEDIUMSPRINGGREEN: Self =
        Self::new(0.0 / 255.0, 250.0 / 255.0, 154.0 / 255.0, 255.0 / 255.0);
    pub const MEDIUMTURQUOISE: Self =
        Self::new(72.0 / 255.0, 209.0 / 255.0, 204.0 / 255.0, 255.0 / 255.0);
    pub const MEDIUMVIOLETRED: Self =
        Self::new(199.0 / 255.0, 21.0 / 255.0, 133.0 / 255.0, 255.0 / 255.0);
    pub const MIDNIGHTBLUE: Self =
        Self::new(25.0 / 255.0, 25.0 / 255.0, 112.0 / 255.0, 255.0 / 255.0);
    pub const MINTCREAM: Self =
        Self::new(245.0 / 255.0, 255.0 / 255.0, 250.0 / 255.0, 255.0 / 255.0);
    pub const MISTYROSE: Self =
        Self::new(255.0 / 255.0, 228.0 / 255.0, 225.0 / 255.0, 255.0 / 255.0);
    pub const MOCCASIN: Self =
        Self::new(255.0 / 255.0, 228.0 / 255.0, 181.0 / 255.0, 255.0 / 255.0);
    pub const NAVAJOWHITE: Self =
        Self::new(255.0 / 255.0, 222.0 / 255.0, 173.0 / 255.0, 255.0 / 255.0);
    pub const NAVY: Self = Self::new(0.0 / 255.0, 0.0 / 255.0, 128.0 / 255.0, 255.0 / 255.0);
    pub const OLDLACE: Self = Self::new(253.0 / 255.0, 245.0 / 255.0, 230.0 / 255.0, 255.0 / 255.0);
    pub const OLIVE: Self = Self::new(128.0 / 255.0, 128.0 / 255.0, 0.0 / 255.0, 255.0 / 255.0);
    pub const OLIVEDRAB: Self =
        Self::new(107.0 / 255.0, 142.0 / 255.0, 35.0 / 255.0, 255.0 / 255.0);
    pub const ORANGE: Self = Self::new(255.0 / 255.0, 165.0 / 255.0, 0.0 / 255.0, 255.0 / 255.0);
    pub const ORANGERED: Self = Self::new(255.0 / 255.0, 69.0 / 255.0, 0.0 / 255.0, 255.0 / 255.0);
    pub const ORCHID: Self = Self::new(218.0 / 255.0, 112.0 / 255.0, 214.0 / 255.0, 255.0 / 255.0);
    pub const PALEGOLDENROD: Self =
        Self::new(238.0 / 255.0, 232.0 / 255.0, 170.0 / 255.0, 255.0 / 255.0);
    pub const PALEGREEN: Self =
        Self::new(152.0 / 255.0, 251.0 / 255.0, 152.0 / 255.0, 255.0 / 255.0);
    pub const PALETURQUOISE: Self =
        Self::new(175.0 / 255.0, 238.0 / 255.0, 238.0 / 255.0, 255.0 / 255.0);
    pub const PALEVIOLETRED: Self =
        Self::new(219.0 / 255.0, 112.0 / 255.0, 147.0 / 255.0, 255.0 / 255.0);
    pub const PAPAYAWHIP: Self =
        Self::new(255.0 / 255.0, 239.0 / 255.0, 213.0 / 255.0, 255.0 / 255.0);
    pub const PEACHPUFF: Self =
        Self::new(255.0 / 255.0, 218.0 / 255.0, 185.0 / 255.0, 255.0 / 255.0);
    pub const PERU: Self = Self::new(205.0 / 255.0, 133.0 / 255.0, 63.0 / 255.0, 255.0 / 255.0);
    pub const PINK: Self = Self::new(255.0 / 255.0, 192.0 / 255.0, 203.0 / 255.0, 255.0 / 255.0);
    pub const PLUM: Self = Self::new(221.0 / 255.0, 160.0 / 255.0, 221.0 / 255.0, 255.0 / 255.0);
    pub const POWDERBLUE: Self =
        Self::new(176.0 / 255.0, 224.0 / 255.0, 230.0 / 255.0, 255.0 / 255.0);
    pub const PURPLE: Self = Self::new(128.0 / 255.0, 0.0 / 255.0, 128.0 / 255.0, 255.0 / 255.0);
    pub const REBECCAPURPLE: Self =
        Self::new(102.0 / 255.0, 51.0 / 255.0, 153.0 / 255.0, 255.0 / 255.0);
    pub const RED: Self = Self::new(255.0 / 255.0, 0.0 / 255.0, 0.0 / 255.0, 255.0 / 255.0);
    pub const ROSYBROWN: Self =
        Self::new(188.0 / 255.0, 143.0 / 255.0, 143.0 / 255.0, 255.0 / 255.0);
    pub const ROYALBLUE: Self =
        Self::new(65.0 / 255.0, 105.0 / 255.0, 225.0 / 255.0, 255.0 / 255.0);
    pub const SADDLEBROWN: Self =
        Self::new(139.0 / 255.0, 69.0 / 255.0, 19.0 / 255.0, 255.0 / 255.0);
    pub const SALMON: Self = Self::new(250.0 / 255.0, 128.0 / 255.0, 114.0 / 255.0, 255.0 / 255.0);
    pub const SANDYBROWN: Self =
        Self::new(244.0 / 255.0, 164.0 / 255.0, 96.0 / 255.0, 255.0 / 255.0);
    pub const SEAGREEN: Self = Self::new(46.0 / 255.0, 139.0 / 255.0, 87.0 / 255.0, 255.0 / 255.0);
    pub const SEASHELL: Self =
        Self::new(255.0 / 255.0, 245.0 / 255.0, 238.0 / 255.0, 255.0 / 255.0);
    pub const SIENNA: Self = Self::new(160.0 / 255.0, 82.0 / 255.0, 45.0 / 255.0, 255.0 / 255.0);
    pub const SILVER: Self = Self::new(192.0 / 255.0, 192.0 / 255.0, 192.0 / 255.0, 255.0 / 255.0);
    pub const SKYBLUE: Self = Self::new(135.0 / 255.0, 206.0 / 255.0, 235.0 / 255.0, 255.0 / 255.0);
    pub const SLATEBLUE: Self =
        Self::new(106.0 / 255.0, 90.0 / 255.0, 205.0 / 255.0, 255.0 / 255.0);
    pub const SLATEGRAY: Self =
        Self::new(112.0 / 255.0, 128.0 / 255.0, 144.0 / 255.0, 255.0 / 255.0);
    pub const SLATEGREY: Self =
        Self::new(112.0 / 255.0, 128.0 / 255.0, 144.0 / 255.0, 255.0 / 255.0);
    pub const SNOW: Self = Self::new(255.0 / 255.0, 250.0 / 255.0, 250.0 / 255.0, 255.0 / 255.0);
    pub const SPRINGGREEN: Self =
        Self::new(0.0 / 255.0, 255.0 / 255.0, 127.0 / 255.0, 255.0 / 255.0);
    pub const STEELBLUE: Self =
        Self::new(70.0 / 255.0, 130.0 / 255.0, 180.0 / 255.0, 255.0 / 255.0);
    pub const TAN: Self = Self::new(210.0 / 255.0, 180.0 / 255.0, 140.0 / 255.0, 255.0 / 255.0);
    pub const TEAL: Self = Self::new(0.0 / 255.0, 128.0 / 255.0, 128.0 / 255.0, 255.0 / 255.0);
    pub const THISTLE: Self = Self::new(216.0 / 255.0, 191.0 / 255.0, 216.0 / 255.0, 255.0 / 255.0);
    pub const TOMATO: Self = Self::new(255.0 / 255.0, 99.0 / 255.0, 71.0 / 255.0, 255.0 / 255.0);
    pub const TURQUOISE: Self =
        Self::new(64.0 / 255.0, 224.0 / 255.0, 208.0 / 255.0, 255.0 / 255.0);
    pub const VIOLET: Self = Self::new(238.0 / 255.0, 130.0 / 255.0, 238.0 / 255.0, 255.0 / 255.0);
    pub const WHEAT: Self = Self::new(245.0 / 255.0, 222.0 / 255.0, 179.0 / 255.0, 255.0 / 255.0);
    pub const WHITE: Self = Self::new(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0);
    pub const WHITESMOKE: Self =
        Self::new(245.0 / 255.0, 245.0 / 255.0, 245.0 / 255.0, 255.0 / 255.0);
    pub const YELLOW: Self = Self::new(255.0 / 255.0, 255.0 / 255.0, 0.0 / 255.0, 255.0 / 255.0);
    pub const YELLOWGREEN: Self =
        Self::new(154.0 / 255.0, 205.0 / 255.0, 50.0 / 255.0, 255.0 / 255.0);

    pub const fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }
}

impl From<RGBAf32> for RGBAu8 {
    fn from(value: RGBAf32) -> Self {
        Self {
            red: (value.red * 255.0) as u8,
            green: (value.green * 255.0) as u8,
            blue: (value.blue * 255.0) as u8,
            alpha: (value.alpha * 255.0) as u8,
        }
    }
}

impl From<RGBAu8> for RGBAf32 {
    fn from(value: RGBAu8) -> Self {
        Self {
            red: value.red as f32 / 255.0,
            green: value.green as f32 / 255.0,
            blue: value.blue as f32 / 255.0,
            alpha: value.alpha as f32 / 255.0,
        }
    }
}
