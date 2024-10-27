use rexiv2::Metadata;
use wallfacer::{config::WallpaperConfig, geometry::Geometry, wallpapers::WallpapersCsv};

fn main() {
    // namespace needs to be registered before using, url doesn't matter
    rexiv2::register_xmp_namespace("http://example.com/wallfacer", "wallfacer")
        .expect("could not register wallfacer namespace");

    let cfg = WallpaperConfig::new();
    let csv = WallpapersCsv::load(&cfg);

    for (_, info) in &csv {
        println!("{}", info.filename);

        let img = cfg.wallpapers_dir.join(&info.filename);
        let meta = Metadata::new_from_path(&img).expect("could not init new metadata");

        // ================== WRITE ==================

        // let _face_arr = info
        //     .faces
        //     .iter()
        //     .map(std::string::ToString::to_string)
        //     .collect_vec();

        // // set face metadata
        // let face_strings = if info.faces.is_empty() {
        //     "[]".to_string()
        // } else {
        //     info.faces
        //         .iter()
        //         .map(std::string::ToString::to_string)
        //         .join(",")
        // };

        // meta.set_tag_string("Xmp.wallfacer.faces", &face_strings)
        //     .unwrap_or_else(|_| panic!("could not set Xmp.wallfacer.faces: {face_strings:?}"));

        // // set crop data
        // for (aspect, geom) in &info.geometries {
        //     let crop_key = format!("Xmp.wallfacer.crop.{}", aspect);
        //     meta.set_tag_string(&crop_key, &geom.to_string())
        //         .unwrap_or_else(|_| panic!("could not set {crop_key}: {geom}"));
        // }

        // meta.save_to_file(&img).expect("could not save metadata");

        // ================== READ ==================

        let read_faces = meta
            .get_tag_string("Xmp.wallfacer.faces")
            .expect("could not get Xmp.wallfacer.faces");

        let read_faces = if read_faces == "[]" {
            Vec::new()
        } else {
            read_faces
                .split(',')
                .map(|face| {
                    face.try_into()
                        .unwrap_or_else(|_| panic!("could not convert face {face} into string"))
                })
                .collect()
        };

        assert_eq!(info.faces, read_faces, "face mismatch");

        for (aspect, geom) in &info.geometries {
            let crop_key = format!("Xmp.wallfacer.crop.{}", aspect);
            let geom_str = meta
                .get_tag_string(&crop_key)
                .unwrap_or_else(|_| panic!("could not get {crop_key}"));
            let read_geom: Geometry = geom_str
                .as_str()
                .try_into()
                .unwrap_or_else(|_| panic!("could not convert face {geom_str} into string"));

            assert_eq!(*geom, read_geom, "geometry mismatch");
        }
    }
}
