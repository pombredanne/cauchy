#define _T(a) L ## a
//#define _T(a) a

static inline int hex2byte(char c) {
	return (c>=_T('0') && c<=_T('9')) ? (c)-_T('0') :
			(c>=_T('A') && c<=_T('F')) ? (c)-_T('A')+10 :
					(c>=_T('a') && c<=_T('f')) ? (c)-_T('a')+10 : 0;
}

static inline int byte2hex(char c) {
	return ((c<10) ? (c)+_T('0') : (c)-10+_T('a'));
}

static inline int is_hex(char c) {
	return ((c>=_T('0') && c<=_T('9')) ||
			(((c>=_T('a') && c<=_T('f'))) ||
					((c>=_T('A') && c<=_T('F')))));
}


unsigned char *bin2hex(unsigned char *bin, unsigned char *hex, int len)
{
	int i;
	for(i = 0; i < len; i++)
	{
		unsigned char byte = *bin++;
		unsigned char hbyte = (byte & 0xF0) >> 4;
		unsigned char lbyte = byte & 0xF;
		*hex++ = byte2hex(hbyte);
		*hex++ = byte2hex(lbyte);
	}
	*hex++ = 0;
	return hex;
}

unsigned char* hex2bin(char *hex, unsigned char *bin, int iLen, int *oLen)
{
	int bLen;
	int i;

	// iLen = (iLen <= 0) ? strlen(hex) : iLen;
	// if(strncmp(hex, "0x", 2) == 0)
	// {
	// 	/* hex string has 0x prefix */
	// 	hex = hex + 2;
	// 	iLen -= 2;
	// }

	if(iLen%2 != 0)
	{
		/* hex string is not a multiple of 2 in length */
		return NULL;
	}

	bLen = iLen / 2;
	memset(bin,0,bLen);

	for(i = 0; i < bLen; i++)
	{
		char hbyte = *hex++;
		char lbyte = *hex++;

		if(!is_hex(hbyte) || !is_hex(lbyte)) {
			/* invalid character */
			return NULL;
		}
		*bin++ = (unsigned char) (hex2byte(hbyte)<<4 | hex2byte(lbyte));
	}

	if(oLen != NULL)
	{
		*oLen = i;
	}
	return bin;
}
