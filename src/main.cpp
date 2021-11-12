#include "serialcxx/serialcxx.hpp"
#include <iostream>

uint32_t add(uint32_t in) {
  return in;
}

//Test with socat -d -d pty,raw,echo=0 pty,raw,echo=0
int main() {
  auto readPort = serialcxx::open_port("/dev/pts/2", 115'000);
  auto port = serialcxx::open_port("/dev/pts/1", 115'000);

  auto str = "Hello\n";
  port->write_str(str);

  std::array<uint8_t, 20> a{};
  auto res = readPort->read(rust::Slice<uint8_t>{a.data(), a.size()});

  if (res.error != serialcxx::SerialError::NoErr) {
    printf("Error!");
  }

  printf("This message was %lu bytes.", res.bytes_read);

  return 0;
}


