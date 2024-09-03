#pragma once

#include <stdlib.h>

int udp_init(void);
int udp_send(const void* payload, size_t length);
