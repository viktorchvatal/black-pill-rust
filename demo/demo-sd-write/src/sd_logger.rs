use core::{fmt::Write};
use arrayvec::ArrayString;
use embedded_hal::{spi::FullDuplex, digital::v2::OutputPin};
use embedded_sdmmc::{
    Controller, SdMmcSpi, TimeSource, VolumeIdx, Volume, Mode, Directory, File, BlockDevice,
};

#[derive(Debug)]
pub enum SdWriteError {
    CannotConnect,
    NoSuitableVolume,
    CannotReadRootDir,
    CannotOpenFile,
    CannotWriteToOpenedFile(ArrayString::<30>),
}

/// Connect to Sd card and append the given `file_data` to the file named
/// `file_name` (file is created if not exists), on the first suitable
/// primary partition (if found) in the card root directory
pub fn append_to_file<SPI, CS, T>(
    controller: &mut Controller<SdMmcSpi<SPI, CS>, T>,
    file_name: &str,
    file_data: &str,
) -> Result<(), SdWriteError>
where
    SPI: FullDuplex<u8>,
    CS: OutputPin,
    T: TimeSource,
    <SPI as FullDuplex<u8>>::Error: core::fmt::Debug
{
    match controller.device().init() {
        Ok(_) => {
            let result = write_to_volume(controller, file_name, file_data);
            controller.device().deinit();
            result
        },
        Err(_err) => Err(SdWriteError::CannotConnect),
    }
}

fn write_to_volume<D, T>(
    controller: &mut Controller<D, T>,
    file_name: &str,
    file_data: &str,
) -> Result<(), SdWriteError>
where D: BlockDevice, T: TimeSource {
    let mut volume = open_volume(controller)?;

    match controller.open_root_dir(&mut volume) {
        Ok(dir) => {
            let result = write_to_file_in_dir(controller, &dir, &mut volume, file_name, file_data);
            controller.close_dir(&mut volume, dir);
            result
        },
        Err(_err) => Err(SdWriteError::CannotReadRootDir),
    }
}

fn write_to_file_in_dir<D, T>(
    controller: &mut Controller<D, T>,
    directory: &Directory,
    volume: &mut Volume,
    file_name: &str,
    file_data: &str,
) -> Result<(), SdWriteError>
where D: BlockDevice, T: TimeSource {
    match controller.open_file_in_dir(
        volume, &directory, file_name, Mode::ReadWriteCreateOrAppend
    ) {
        Ok(mut file) => {
            let result = write_to_opened_file(controller, volume, &mut file, file_data);
            let _ = controller.close_file(volume, file);
            result
        }
        Err(_err) => Err(SdWriteError::CannotOpenFile),
    }
}

fn write_to_opened_file<D, T>(
    controller: &mut Controller<D, T>,
    volume: &mut Volume,
    file: &mut File,
    file_data: &str,
) -> Result<(), SdWriteError>
where D: BlockDevice, T: TimeSource {
    match controller.write(volume, file, file_data.as_bytes()) {
        Ok(_) => Ok(()),
        Err(error) => {
            let mut message = ArrayString::<30>::new();
            let _  = write!(&mut message, "{:?}", error);
            Err(SdWriteError::CannotWriteToOpenedFile(message))
        },
    }
}

fn open_volume<D, T>(
    controller: &mut Controller<D, T>,
) -> Result<Volume, SdWriteError>
where D: BlockDevice, T: TimeSource {
    for volume_index in 0..4 {
        if let Ok(volume) = controller.get_volume(VolumeIdx(volume_index)) {
            return Ok(volume);
        }
    }

    return  Err(SdWriteError::NoSuitableVolume);
}