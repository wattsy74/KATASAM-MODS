SECTIONS
{
  .boot2 ORIGIN(BOOT2) :
  {
    KEEP(*(.boot2));
  } > BOOT2
}

ASSERT(SIZEOF(.boot2) == 256, "ERROR: Boot2 must be exactly 256 bytes");
