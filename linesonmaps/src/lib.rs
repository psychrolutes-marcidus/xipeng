// pub mod linestring;
pub mod types;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytemuck::bytes_of;
    use geo_traits::GeometryType;
    use geo_traits::*;
    use wkb::reader::read_wkb;

    #[test]
    #[ignore = "just fooling around"]
    fn three_dimensional() {
        // https://en.wikipedia.org/wiki/Well-known_text_representation_of_geometry
        const X: f64 = 1.0;
        const XX: f64 = 1.1;
        const Y: f64 = 2.0;
        const YY: f64 = 2.2;
        const M: f64 = 4.0;
        const MM: f64 = 4.4;
        const NDR: u8 = 1;
        const LINESTRINGM: u32 = 2002;

        // const _: () = assert_eq!(1,2);
        const NDR_LE: [u8; 1] = NDR.to_le_bytes();
        const LINESTRINGM_LE: [u8; 4] = LINESTRINGM.to_le_bytes();
        const X_LE: [u8; 8] = X.to_le_bytes();
        const XX_LE: [u8; 8] = XX.to_le_bytes();
        const Y_LE: [u8; 8] = Y.to_le_bytes();
        const YY_LE: [u8; 8] = YY.to_le_bytes();
        const M_LE: [u8; 8] = M.to_le_bytes();
        const MM_LE: [u8; 8] = MM.to_le_bytes();
        // println!("{NDR_LE:?}\n{LINESTRINGM_LE:?}\n{X_LE:?}\n{Y_LE:?}\n{M_LE:?}");
        let wkb = [
            NDR_LE.to_vec(),
            LINESTRINGM_LE.to_vec(),
            X_LE.to_vec(),
            Y_LE.to_vec(),
            M_LE.to_vec(),
            XX_LE.to_vec(),
            YY_LE.to_vec(),
            MM_LE.to_vec(),
        ]
        .concat();
        let parsed = read_wkb(&wkb).unwrap();
        assert_eq!(
            parsed.geometry_type(),
            wkb::reader::GeometryType::LineString
        );
        assert_eq!(parsed.dim(), Dimensions::Xym);
        let t = match parsed.as_type() {
            GeometryType::LineString(ls) => ls,
            _ => unreachable!(),
        };
        println!("{0:?}",t.coord(0).unwrap());
        assert_eq!(t.num_coords(), 2); // does not quite work for some reason
        // assert_eq!(parsed.num)

        // assert!(false);
    }
    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
