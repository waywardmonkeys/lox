use std::{
    convert::{TryInto},
    fs::File,
    io::{Seek, Read, SeekFrom, BufWriter},
    time::Instant,
};

use failure::{bail, err_msg, format_err, Error, ResultExt};
use structopt::StructOpt;
use term_painter::{ToStyle, Color};

use lox::{
    prelude::*,
    fat::AnyMesh,
    io::{
        FileFormat, FileEncoding,
        stl, ply,
    },
};


mod opt;
mod util;

use crate::{
    opt::Opt,
    util::encoding_str,
};


macro_rules! print {
    ($($t:tt)*) => {{
        use std::io::{self, Write};

        std::print!($($t)*);
        // If an error occurs here... oh well.
        let _ = io::stdout().flush();
    }}
}

/// We just catch potential errors here and pretty print them. The actual
/// useful code is in `run()`.
fn main() {
    if let Err(e) = run() {
        println!("An error occured: {}", e);

        for cause in e.iter_causes() {
            println!("  ... caused by: {}", cause);
        }

        if std::env::var("RUST_BACKTRACE") == Ok("1".to_string()) {
            println!();
            println!("{}", e.backtrace());
        }

        std::process::exit(1);
    }
}

fn run() -> Result<(), Error> {
    let opt = Opt::from_args();

    let start_time = Instant::now();

    let before_load = Instant::now();
    let mesh_data = load_file(&opt).context("could not read source file")?;
    let load_time = before_load.elapsed();

    print_mesh_info(&mesh_data);

    let before_write = Instant::now();
    write_file(&opt, &mesh_data).context("could not write target file")?;
    let write_time = before_write.elapsed();

    println!(
        "{}: {:.2?} ({:.2?} loading, {:.2?} writing)",
        Color::Blue.bold().paint("⟨ℹ⟩ Processing time"),
        start_time.elapsed(),
        load_time,
        write_time,
    );

    Ok(())
}

fn print_mesh_info(mesh_data: &AnyMesh) {
    println!("{}", Color::Green.bold().paint("⟨ℹ⟩ Mesh information:"));

    // ===== Vertex Infos ====================================================
    // Collect vertex properties
    let mut vertex_props = vec![];
    if mesh_data.vertex_positions.is_some() {
        vertex_props.push("position");
    }

    let vertex_props = if vertex_props.is_empty() {
        "none".to_string()
    } else {
        let mut out = vertex_props[0].to_string();
        for prop in &vertex_props[1..] {
            out += ", ";
            out += prop;
        }
        out
    };

    println!(
        "    {} vertices (properties: {})",
        mesh_data.mesh.num_vertices(),
        vertex_props,
    );


    // ===== Face Infos ======================================================
    println!(
        "    {} faces (properties: none)",
        mesh_data.mesh.num_faces(),
    );
}

fn load_file(opt: &Opt) -> Result<AnyMesh, Error> {
    let mut file = File::open(&opt.source).context("failed to open file")?;

    // Figure out the file format
    let file_format = {
        let format = opt.source_format.or_else(|| FileFormat::from_extension(&opt.source));
        let format = match format {
            Some(format) => Some(format),
            None => {
                let mut start = Vec::new();
                file.by_ref().take(1024).read_to_end(&mut start)?;
                file.seek(SeekFrom::Start(0))?;

                FileFormat::from_file_start(&start)
            }
        };

        format.ok_or_else(|| err_msg(
            "couldn't determine source file format, please specify it explicitly using \
                '--source-format'"
        ))?
    };


    // Parse the header of the file, print some information and return the
    // abstract reader object.
    macro_rules! get_reader_and_print_info {
        ($($variant:ident => $module:ident,)*) => {
            match file_format {
                $(
                    FileFormat::$variant => {
                        let reader = $module::Reader::new(file)
                            .context(format!("failed to reader {} header", FileFormat::$variant))?;

                        println!(
                            "{}: {} ({} encoding)",
                            Color::Blue.bold().paint("⟨ℹ⟩ Source format"),
                            file_format,
                            encoding_str(FileEncoding::from(reader.encoding())),
                        );

                        Box::new(reader) as Box<dyn DynStreamSource<_>>
                    }
                )*
                _ => bail!(
                    "File format '{}' not supported (this is probably a bug)",
                    file_format,
                ),
            }
        }
    }

    let reader = get_reader_and_print_info!(
        Ply => ply,
        Stl => stl,
    );


    // Read from the reader into an `AnyMesh`
    print!("⟨￫⟩ Reading source ...");
    let mut mesh = AnyMesh::empty();
    reader.transfer_to(&mut mesh)?;
    mesh.finish()?;
    println!(" done");

    Ok(mesh)
}

fn write_file(opt: &Opt, data: &AnyMesh) -> Result<(), Error> {
    // Figure out the file format
    let file_format = opt.target_format
        .or_else(|| FileFormat::from_extension(&opt.target))
        .ok_or_else(|| err_msg(
            "couldn't determine target file format, please specify it explicitly using \
                '--target-format'"
        ))?;


    let encoding = opt.target_encoding.encoding_for(file_format).ok_or(
        format_err!(
            "the encoding {:?} is not supported by the {} format",
            opt.target_encoding,
            file_format,
        )
    )?;

    println!(
        "{}: {} ({} encoding)",
        Color::Blue.bold().paint("⟨ℹ⟩ Target format"),
        file_format,
        encoding_str(encoding),
    );

    let file = BufWriter::new(File::create(&opt.target)?);

    match file_format {
        FileFormat::Ply => {
            unimplemented!()
        }
        FileFormat::Stl => {
            print!("⟨￩⟩ Writing mesh ...");

            stl::Config::new(encoding.try_into().unwrap())
                .into_writer(file)
                .transfer_from(data)?;

            println!(" done");
        }
        _ => bail!("File format '{}' not supported", file_format),
    }

    Ok(())
}
