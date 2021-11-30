#include "serialcxx/serialcxx.hpp"
#include <iostream>

//Test with socat -d -d pty,raw,echo=0 pty,raw,echo=0
int main() {
  auto readPort = serialcxx::open_port("/dev/pts/2", 115'000);
  auto port = serialcxx::open_port("/dev/pts/3", 115'000);

  port->set_timeout(2.5);
  readPort->set_timeout(2.5);

  auto str = "Hello From C++!\n";
  port->write_str(str);

  std::string buff{};
  auto res = readPort->read_line(buff);

  if (res.error != serialcxx::SerialError::NoErr) {
    printf("Error!");
  }

  printf("This message was %lu bytes. \n", res.bytes_read);
  printf("This message was '%s'", buff.c_str());

  return 0;
}


