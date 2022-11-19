use core::{fmt::Write};
use arrayvec::ArrayString;
use embedded_hal::{spi::FullDuplex, digital::v2::OutputPin};
use embedded_sdmmc::{
    Controller, SdMmcSpi, TimeSource, VolumeIdx, Volume, Mode, Directory, File,
};

#[derive(Debug)]
pub enum SdWriteError {
    CannotConnect,
    NoSuitableVolume,
    CannotReadRootDir,
    CannotOpenFile,
    CannotWriteToOpenedFile(ArrayString::<30>),
}

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
    let mut volume = open_volume(controller)?;

    let result = match controller.open_root_dir(&mut volume) {
        Ok(dir) => write_to_file_in_dir(controller, dir, &mut volume, file_name, file_data),
        Err(_err) => Err(SdWriteError::CannotReadRootDir),
    };

    controller.device().deinit();
    result
}

fn write_to_file_in_dir<SPI, CS, T>(
    controller: &mut Controller<SdMmcSpi<SPI, CS>, T>,
    directory: Directory,
    volume: &mut Volume,
    file_name: &str,
    file_data: &str,
) -> Result<(), SdWriteError>
where
    SPI: FullDuplex<u8>,
    CS: OutputPin,
    T: TimeSource,
    <SPI as FullDuplex<u8>>::Error: core::fmt::Debug
{
    let result = match controller.open_file_in_dir(
        volume, &directory, file_name, Mode::ReadWriteCreateOrAppend
    ) {
        Ok(file) => write_to_opened_file(controller, volume, file, file_data),
        Err(_err) => Err(SdWriteError::CannotOpenFile),
    };

    controller.close_dir(volume, directory);
    result
}

fn write_to_opened_file<SPI, CS, T>(
    controller: &mut Controller<SdMmcSpi<SPI, CS>, T>,
    volume: &mut Volume,
    mut file: File,
    file_data: &str,
) -> Result<(), SdWriteError>
where
    SPI: FullDuplex<u8>,
    CS: OutputPin,
    T: TimeSource,
    <SPI as FullDuplex<u8>>::Error: core::fmt::Debug
{
    let result = match controller.write(volume, &mut file, file_data.as_bytes()) {
        Ok(_) => Ok(()),
        Err(error) => {
            let mut message = ArrayString::<30>::new();
            let _  = write!(&mut message, "{:?}", error);
            Err(SdWriteError::CannotWriteToOpenedFile(message))
        },
    };

    let _ = controller.close_file(volume, file);
    result
}


fn open_volume<SPI, CS, T>(
    controller: &mut Controller<SdMmcSpi<SPI, CS>, T>,
) -> Result<Volume, SdWriteError>
where
    SPI: FullDuplex<u8>,
    CS: OutputPin,
    T: TimeSource,
    <SPI as FullDuplex<u8>>::Error: core::fmt::Debug
{
    match controller.device().init() {
        Ok(_) => {
            for volume_index in 0..3 {
                if let Ok(volume) = controller.get_volume(VolumeIdx(volume_index)) {
                    return Ok(volume);
                }
            }

            return  Err(SdWriteError::NoSuitableVolume);
        }
        Err(_err) => Err(SdWriteError::CannotConnect),
    }
}