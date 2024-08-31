#pragma once

#include <stdlib.h>

int udp_init();
int udp_send(const void* payload, size_t length);
