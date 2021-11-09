#include "serialcxx/serialcxx.hpp"
#include <iostream>

uint32_t add(uint32_t in) {
  return in;
}

int main() {
  auto port = serialcxx::open_port("asd",115000);
  serialcxx::add_read_callback(port.into_raw(), &add);
  return 0;
}


