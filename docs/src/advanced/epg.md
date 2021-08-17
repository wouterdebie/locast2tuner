# Electronic Programming Guide
`locast2tuner` also provides Electronic Programming Guide (EPG) information from locast.org. This is served in the [XMLTV](http://wiki.xmltv.org/) format. Emby and PMS both have support for XMLTV which can be used by adding `http://IP:PORT/epg.xml`  (defaults to `http://127.0.0.1:6077/epg.xml`) as an XMLTV TV Guide Data Provider.

In case [Multiplexing](./multiplexing.md) is used, all EPG data is multiplexed as well.
