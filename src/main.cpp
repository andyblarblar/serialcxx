#include "serialcxx/serialcxx.hpp"
#include <iostream>

void testCall(void *userdata, const char *str, uintptr_t size) {
  printf("Read '%s' from the listener!", str);
}

//Test with socat -d -d pty,raw,echo=0 pty,raw,echo=0
int main() {
  //Open ports
  auto readPort = serialcxx::open_port("/dev/pts/3", 115'000);
  auto port = serialcxx::open_port("/dev/pts/4", 115'000);

  //Change port settings
  port->set_timeout(2.5);
  readPort->set_timeout(2.5);

  //Create a listener
  auto builder = readPort->create_listener_builder();
  serialcxx::add_read_callback(builder->self_ptr(), nullptr, &testCall);
  auto listener = builder->build();
  listener->listen();

  //Send String
  auto str = "Hello From C++!\n";
  port->write_str(str);

  //Read String
  std::string buff{};
  auto res = readPort->read_line(buff);

  //Error checking
  if (res.error != serialcxx::SerialError::NoErr) {
    printf("Error!");
  }

  printf("This message was %lu bytes. \n", res.bytes_read);
  printf("This message was '%s' \n", buff.c_str());

  //Send String again, should be in listener
  auto str2 = "This should be in the listener\n!";
  port->write_str(str2);

  for (int i = 0; i < 1000000000; ++i) {
  }

  return 0;
}


