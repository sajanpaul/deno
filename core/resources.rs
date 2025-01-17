// Copyright 2018-2019 the Deno authors. All rights reserved. MIT license.

// Think of Resources as File Descriptors. They are integers that are allocated by
// the privileged side of Deno to refer to various rust objects that need to be
// referenced between multiple ops. For example, network sockets are resources.
// Resources may or may not correspond to a real operating system file
// descriptor (hence the different name).

use downcast_rs::Downcast;
use std;
use std::any::Any;
use std::collections::HashMap;

/// ResourceId is Deno's version of a file descriptor. ResourceId is also referred
/// to as rid in the code base.
pub type ResourceId = u32;

/// These store Deno's file descriptors. These are not necessarily the operating
/// system ones.
type ResourceMap = HashMap<ResourceId, Box<dyn Resource>>;

#[derive(Default)]
pub struct ResourceTable {
  // TODO(bartlomieju): remove pub modifier, it is used by
  // `get_file` method in CLI
  pub map: ResourceMap,
  next_id: u32,
}

impl ResourceTable {
  pub fn get<T: Resource>(&self, rid: ResourceId) -> Option<&T> {
    if let Some(resource) = self.map.get(&rid) {
      return resource.downcast_ref::<T>();
    }

    None
  }

  pub fn get_mut<T: Resource>(&mut self, rid: ResourceId) -> Option<&mut T> {
    if let Some(resource) = self.map.get_mut(&rid) {
      return resource.downcast_mut::<T>();
    }

    None
  }

  // TODO: resource id allocation should probably be randomized for security.
  fn next_rid(&mut self) -> ResourceId {
    let next_rid = self.next_id;
    self.next_id += 1;
    next_rid as ResourceId
  }

  pub fn add(&mut self, resource: Box<dyn Resource>) -> ResourceId {
    let rid = self.next_rid();
    let r = self.map.insert(rid, resource);
    assert!(r.is_none());
    rid
  }

  pub fn entries(&self) -> Vec<(ResourceId, String)> {
    self
      .map
      .iter()
      .map(|(key, value)| (*key, value.inspect_repr().to_string()))
      .collect()
  }

  // close(2) is done by dropping the value. Therefore we just need to remove
  // the resource from the RESOURCE_TABLE.
  pub fn close(&mut self, rid: ResourceId) -> Option<()> {
    if let Some(resource) = self.map.remove(&rid) {
      resource.close();
      return Some(());
    }

    None
  }
}

/// Abstract type representing resource in Deno.
pub trait Resource: Downcast + Any + Send {
  /// Method that allows to cleanup resource.
  fn close(&self) {}

  fn inspect_repr(&self) -> &str;
}
impl_downcast!(Resource);
