#include "serialcxx/src/lib.rs.h"
#include <iostream>
#include "/home/andy/CLionProjects/serialcxx/src/serialcxx/generated/bindings.hpp"

uint32_t add(uint32_t in) {
  return in;
}

int main() {
  auto port = open_port("asd",115000);
  add_read_callback(port.into_raw(), &add);
  return 0;
}


