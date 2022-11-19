use core::{fmt::{Debug, Display}};
use embedded_hal::{spi::FullDuplex, digital::v2::OutputPin};
use embedded_sdmmc::{
    Controller, SdMmcSpi, TimeSource, VolumeIdx, Volume, Mode, Directory, File, BlockDevice,
    Error, SdMmcError
};

pub enum SdWriteError<E>
where E: core::fmt::Debug {
    CannotConnect(SdMmcError),
    NoSuitableVolume,
    CannotReadRootDir(Error<E>),
    CannotOpenFile(Error<E>),
    CannotWriteToOpenedFile(Error<E>),
}

impl<T> Display for SdWriteError<T> where T: Debug {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SdWriteError::CannotConnect(ref err)
                => write!(f, "Conn:{}", device_error_to_str(err)),
            SdWriteError::NoSuitableVolume
                => write!(f, "NoVol"),
            SdWriteError::CannotReadRootDir(ref err)
                => write!(f, "RootE:{}", controller_error_to_str(err)),
            SdWriteError::CannotOpenFile(ref err)
                => write!(f, "OpenE:{}", controller_error_to_str(err)),
            SdWriteError::CannotWriteToOpenedFile(ref err)
                => write!(f, "WrE:{}", controller_error_to_str(err)),
        }
    }
}

/// Connect to Sd card and append the given `file_data` to the file named
/// `file_name` (file is created if not exists), on the first suitable
/// primary partition (if found) in the card root directory
pub fn append_to_file<SPI, CS, T>(
    controller: &mut Controller<SdMmcSpi<SPI, CS>, T>,
    file_name: &str,
    file_data: &str,
) -> Result<(), SdWriteError<SdMmcError>>
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
        Err(error) => Err(SdWriteError::CannotConnect(error)),
    }
}

fn device_error_to_str(error: &SdMmcError) -> &'static str {
    match error {
        SdMmcError::Transport => "Transport",
        SdMmcError::CantEnableCRC => "EnableCrc",
        SdMmcError::TimeoutReadBuffer => "TOReadBuf",
        SdMmcError::TimeoutWaitNotBusy => "TOWaitNoBusy",
        SdMmcError::TimeoutCommand(_) => "TOCommand",
        SdMmcError::TimeoutACommand(_) => "TOACommand",
        SdMmcError::Cmd58Error => "Cmd58Err",
        SdMmcError::RegisterReadError => "RegReadErr",
        SdMmcError::CrcError(_, _) => "Crc",
        SdMmcError::ReadError => "ReadErr",
        SdMmcError::WriteError => "WriteErr",
        SdMmcError::BadState => "BadState",
        SdMmcError::CardNotFound => "CardNotFound",
        SdMmcError::GpioError => "GpioErr",
    }
}

fn controller_error_to_str<E>(error: &Error<E>)-> &'static str where E: Debug {
    match error {
        Error::DeviceError(_) => "DevErr",
        Error::FormatError(_) => "FormatErr",
        Error::NoSuchVolume => "NoVol",
        Error::FilenameError(_) => "FNameErr",
        Error::TooManyOpenDirs => "ManyOpenDirs",
        Error::TooManyOpenFiles => "ManyOpenFiles",
        Error::FileNotFound => "FileNotFound",
        Error::FileAlreadyOpen => "FAlreadyOpen",
        Error::DirAlreadyOpen => "DirAlreadyOpen",
        Error::OpenedDirAsFile => "OpenDirAsFile",
        Error::Unsupported => "Unsupported",
        Error::EndOfFile => "EOF",
        Error::BadCluster => "BadCluster",
        Error::ConversionError => "ConvertErr",
        Error::NotEnoughSpace => "NoSpace",
        Error::AllocationError => "AllocErr",
        Error::JumpedFree => "JumpedFree",
        Error::ReadOnly => "ReadOnly",
        Error::FileAlreadyExists => "FileExists",
    }
}

fn write_to_volume<D, T, E>(
    controller: &mut Controller<D, T>,
    file_name: &str,
    file_data: &str,
) -> Result<(), SdWriteError<E>>
where D: BlockDevice<Error = E>, T: TimeSource, E: Debug {
    let mut volume = open_volume(controller)?;

    match controller.open_root_dir(&mut volume) {
        Ok(dir) => {
            let result = write_to_file_in_dir(controller, &dir, &mut volume, file_name, file_data);
            controller.close_dir(&mut volume, dir);
            result
        },
        Err(error) => Err(SdWriteError::CannotReadRootDir(error)),
    }
}

fn write_to_file_in_dir<D, T, E>(
    controller: &mut Controller<D, T>,
    directory: &Directory,
    volume: &mut Volume,
    file_name: &str,
    file_data: &str,
) -> Result<(), SdWriteError<E>>
where D: BlockDevice<Error = E>, T: TimeSource, E: Debug {
    match controller.open_file_in_dir(
        volume, &directory, file_name, Mode::ReadWriteCreateOrAppend
    ) {
        Ok(mut file) => {
            let result = write_to_opened_file(controller, volume, &mut file, file_data);
            let _ = controller.close_file(volume, file);
            result
        }
        Err(error) => Err(SdWriteError::CannotOpenFile(error)),
    }
}

fn write_to_opened_file<D, T, E>(
    controller: &mut Controller<D, T>,
    volume: &mut Volume,
    file: &mut File,
    file_data: &str,
) -> Result<(), SdWriteError<E>>
where D: BlockDevice<Error = E>, T: TimeSource, E: Debug {
    match controller.write(volume, file, file_data.as_bytes()) {
        Ok(_) => Ok(()),
        Err(error) => Err(SdWriteError::CannotWriteToOpenedFile(error)),
    }
}

fn open_volume<D, T, E>(
    controller: &mut Controller<D, T>,
) -> Result<Volume, SdWriteError<E>>
where D: BlockDevice<Error = E>, T: TimeSource, E: Debug {
    for volume_index in 0..4 {
        if let Ok(volume) = controller.get_volume(VolumeIdx(volume_index)) {
            return Ok(volume);
        }
    }

    return  Err(SdWriteError::NoSuitableVolume);
}