use crate::{Address, AddressType, SERVICE_NAME, TIMEOUT, device::Device, bluetooth_utils::Modalias};
use crate::bluetooth_le_advertising_data::BluetoothAdvertisingData;
use crate::session::Session;
use crate::{Result, Error, device};
use dbus::{Path, nonblock::{Proxy, SyncConnection, stdintf::org_freedesktop_dbus::ObjectManager}};
use std::{collections::HashMap, fmt::Formatter, sync::Arc, u32};
use std::fmt::Debug;

pub(crate) const INTERFACE: &str = "org.bluez.Adapter1";
pub(crate) const PREFIX: &str = "/org/bluez/";

/// Interface to a Bluetooth adapter.
#[derive(Clone)]
pub struct Adapter<'a> {
    session: &'a Session,
    proxy: Proxy<'static, &'a SyncConnection>,
    name: Arc<String>,
}

impl<'a> Debug for Adapter<'a> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Adapter {{ session: {:?}, name: {} }}", self.session(), self.name())
    }
}

impl<'a> Adapter<'a> {

    /// Create Bluetooth adapter interface for adapter with specified name.
    pub(crate) fn new(session: &'a Session, name: &str) -> Self {
        let path = PREFIX.to_string() + name;
        Self {
            session,
            proxy: Proxy::new(SERVICE_NAME, path, TIMEOUT, session.connection()),
            name: Arc::new(name.to_string())
        }
    }

    /// The Bluetooth adaper D-Bus path.
    ///
    /// For example: /org/bluez/hci0
    pub(crate) fn dbus_path(&self) -> &Path {
        &self.proxy.path
    }

    /// The Bluetooth adapter name.
    ///
    /// For example hci0.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Bluetooth session.
    pub fn session(&self) -> &Session {
        self.session
    }

    // pub fn get_id(&self) -> String {
    //     self.object_path.clone()
    // }

    // pub async fn get_first_device(&self) -> Result<BluetoothDevice<'_>> {
    //     let devices = bluetooth_utils::list_devices(&self.session.get_connection(), &self.object_path).await?;
    //
    //     if devices.is_empty() {
    //         return Err(Box::from("No device found."));
    //     }
    //     Ok(BluetoothDevice::new(self.session, &devices[0]))
    // }

    // pub async fn get_addata(&self) -> Result<BluetoothAdvertisingData<'_>> {
    //     let addata = bluetooth_utils::list_addata_1(&self.session.get_connection(), &self.object_path).await?;
    //
    //     if addata.is_empty() {
    //         return Err(Box::from("No addata found."));
    //     }
    //     Ok(BluetoothAdvertisingData::new(&self.session, &addata[0]))
    // }

    /// Bluetooth addresses of discovered Bluetooth devices.
    pub async fn device_addresses(&self) -> Result<Vec<Address>> {
        let prefix = format!("{}/dev_", self.dbus_path());
        let mut addrs = Vec::new();
        let p = Proxy::new(SERVICE_NAME, "/", TIMEOUT, self.session().connection());
        for (path, interfaces) in p.get_managed_objects().await? {
            match path.strip_prefix(&prefix) {
                Some(addr) if interfaces.contains_key(device::INTERFACE) => {
                    let addr = addr.replace('_', ":");
                    let addr: Address = addr.parse()?;
                    addrs.push(addr);
                }
                _ => (),
            }
        }
        Ok(addrs)
    }

    /// Get interface to Bluetooth device of specified address.
    pub fn device(&self, address: Address) -> Device {
        Device::new(self.session(), self.name.clone(), address)
    }

    dbus_interface!(INTERFACE);

    // ===========================================================================================
    // Properties
    // ===========================================================================================

    define_property!(
        /// The Bluetooth device address.
        address, "Address" => String
    );

    /// The Bluetooth Address Type.
    ///
    /// For dual-mode and BR/EDR
    /// only adapter this defaults to "public". Single mode LE
    /// adapters may have either value. With privacy enabled
    /// this contains type of Identity Address and not type of
    /// address used for connection.
    pub async fn address_type(&self) -> Result<AddressType> {
        let address_type: String = self.get_property("AddressType").await?;
        Ok(address_type.parse()?)
    }

    define_property!(
        ///	The Bluetooth system name (pretty hostname).
        /// 
        /// This property is either a static system default
        /// or controlled by an external daemon providing
        /// access to the pretty hostname configuration.
        system_name, "Name" => String
    );

    define_property!(
        /// The Bluetooth friendly name. 
        ///
        /// This value can be changed.
        /// 
        /// In case no alias is set, it will return the system
        /// provided name. Setting an empty string as alias will
        /// convert it back to the system provided name.
        /// 
        /// When resetting the alias with an empty string, the
        /// property will default back to system name.
        /// 
        /// On a well configured system, this property never
        /// needs to be changed since it defaults to the system
        /// name and provides the pretty hostname. Only if the
        /// local name needs to be different from the pretty
        /// hostname, this property should be used as last
        /// resort.
        alias, set_alias, "Alias" => String
    );

    define_property!(
        /// The Bluetooth class of device.
        ///
		///	This property represents the value that is either
		///	automatically configured by DMI/ACPI information
		///	or provided as static configuration.
        class, "Class" => u32
    );

    define_property!(
        /// Switch an adapter on or off. This will also set the
        /// appropriate connectable state of the controller.
        /// 
        /// The value of this property is not persistent. After
        /// restart or unplugging of the adapter it will reset
        /// back to false.
        is_powered, set_powered, "Powered" => bool
    );

    define_property!(
        /// Switch an adapter to discoverable or non-discoverable
        /// to either make it visible or hide it. 
        /// 
        /// This is a global
        /// setting and should only be used by the settings
        /// application.
        /// 
        /// If the DiscoverableTimeout is set to a non-zero
        /// value then the system will set this value back to
        /// false after the timer expired.
        /// 
        /// In case the adapter is switched off, setting this
        /// value will fail.
        /// 
        /// When changing the Powered property the new state of
        /// this property will be updated via a PropertiesChanged
        /// signal.
        /// 
        /// For any new adapter this settings defaults to false.        
        is_discoverable, set_discoverable, "Discoverable" => bool
    );

    define_property!(
        /// Switch an adapter to pairable or non-pairable. 
        ///
        /// This is
        /// a global setting and should only be used by the
        /// settings application.
        /// 
        /// Note that this property only affects incoming pairing
        /// requests.
        /// 
        /// For any new adapter this settings defaults to true.        
        is_pairable, set_pairable, "Pairable" => bool
    );

    define_property!(
        /// The pairable timeout in seconds. 
        ///
        /// A value of zero
        /// means that the timeout is disabled and it will stay in
        /// pairable mode forever.
        /// 
        /// The default value for pairable timeout should be
        /// disabled (value 0).
        pairable_timeout, set_pairable_timeout, "PairableTimeout" => u32
    );

    define_property!(
        /// The discoverable timeout in seconds. 
        ///
        /// A value of zero
        /// means that the timeout is disabled and it will stay in
        /// discoverable/limited mode forever.
        /// 
        /// The default value for the discoverable timeout should
        /// be 180 seconds (3 minutes).
        discoverable_timeout, set_discoverable_timeout, "DiscoverableTimeout" => u32
    );

    define_property!(
        ///	Indicates that a device discovery procedure is active.
        is_discovering, "Discovering" => bool
    );

    define_property!(
        /// List of 128-bit UUIDs that represents the available
		/// lcal services.
        uuids, "UUIDs" => Vec<String>
    );

    /// Local Device ID information in modalias format
	/// used by the kernel and udev.
    pub async fn modalias(&self) -> Result<Modalias> {
        let modalias: String = self.get_property("Modalias").await?;
        Ok(modalias.parse()?)
    }

    // ===========================================================================================
    // Methods
    // ===========================================================================================

    // http://git.kernel.org/cgit/bluetooth/bluez.git/tree/doc/adapter-api.txt#n12
    // Don't use this method, it's just a bomb now.
    //pub fn start_discovery(&self) -> Result<()> {
    //    Err(Box::from("Deprecated, use Discovery Session"))
    //}

    // http://git.kernel.org/cgit/bluetooth/bluez.git/tree/doc/adapter-api.txt#n27
    // Don't use this method, it's just a bomb now.
    //pub fn stop_discovery(&self) -> Result<()> {
    //    Err(Box::from("Deprecated, use Discovery Session"))
    //}

    /// This removes the remote device object at the given
    /// path.
    ///
    /// It will remove also the pairing information.
    pub async fn remove_device(&self, device: &str) -> Result<()> {
        self.call_method("RemoveDevice", (Path::from(device),)).await?;
        Ok(())
    }

    /// This method connects to device without need of
    /// performing General Discovery. 
    ///
    /// Connection mechanism is
    /// similar to Connect method from Device1 interface with
    /// exception that this method returns success when physical
    /// connection is established. After this method returns,
    /// services discovery will continue and any supported
    /// profile will be connected. There is no need for calling
    /// Connect on Device1 after this call. If connection was
    /// successful this method returns object path to created
    /// device object.
    ///
    /// Parameters that may be set in the filter dictionary
    /// include the following:    
    ///
    ///  `address` -
    ///     The Bluetooth device address of the remote
    ///     device. This parameter is mandatory.
    ///
    /// `address_type` -
    ///     The Bluetooth device Address Type. This is
    ///     address type that should be used for initial
    ///     connection. If this parameter is not present
    ///     BR/EDR device is created.    
    pub async fn connect_device(
        &self, address: Address, address_type: Option<AddressType>,
    ) -> Result<Path<'static>> {
        let mut m = HashMap::new();
        m.insert("Address", address.to_string());
        if let Some(address_type) = address_type {
            m.insert("AddressType", address_type.to_string());
        }
        let (path,): (Path,) = self.call_method("ConnectDevice", (m,)).await?;
        Ok(path)
    }
}
