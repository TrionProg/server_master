About
=====
This is an example of server, what uses 5 DBs at same time. This version is public as an example. License:MIT.

О проекте
=========

Вокруг игры должно быть сообщество, а следовательно, необходимо реализовать форум, позволяющий выдерживать большую нагрузку(получается небольшая соц-сеть).

Требования и особенности:
-------------------------
* Использование тех видов БД, которые наиболее уместны
* Максимальная распределённость, поэтому всё держится на uuid
* Акцент на скорость
* Кэширование с помощью redis
* Развитое API уровня create_user, create_thread
* Сравнительно небольшое количество сложных SQL запросов ввиду того, что в проекте соединены различные БД

СУБД
----

Cassandra для постов, потому что их много, они хранятся на нескольких серверах/узлах и читаются достаточно редко.
Mongo для хранения детальной информации о пользователе, потому что она весьма ветвистая и содержит множество списков.
Postgres для хранения краткой информации о пользователе, и веток форума(планируется заменить на специальный списковый движок, ибо каждая ветка по сути является списком UUID постов).
Redis в качестве кэша. Кэширование позволило многократно увеличить производительность

![](https://github.com/TrionProg/server_master/blob/master/cache.png)

Neo4j для социальных связей. Только в качестве эксперимента. Очень медленно заполняется данными!

Для визуализации работы прилагается простой web-интерфейс на iron. С помощью этого web-интерфейс и были проведены замеры производительности различных вариантов сервера(файлы server_master_both_caching, server_master_cassandra_caching и тд).

[Схема БД](https://github.com/TrionProg/server_master/blob/master/DB_diagram.pdf)

Подробнее
---------

* [forum.rs](https://github.com/TrionProg/server_master/blob/master/src/db/forum.rs).
Все посты хранятся в cassandra, а для составления списка постов используется таблица posts в postgres, поэтому получение списков постов весьма медленная операция, это временная мера. Посты кэшируются.
* [images.rs](https://github.com/TrionProg/server_master/blob/master/src/db/images.rs).
Это хранилище картинок. Временно они хранятся в postgres, конечно они должны храниться в cassandra. Они так же кэшируются
* [users.rs](https://github.com/TrionProg/server_master/blob/master/src/db/users.rs).
Это большое API для управления пользователями. Информация о пользователе разбита на две части: короткую и полную. Короткая содержит лишь логин, uuid маленькой аватарки и рейтинг. При "сборке" треда форума очень много обращений именно за такой информацией, поэтому эта информация кэшируется путем сериализации в двоичный формат. Постоянно полная информация(эквивалентна стене в соц. сети) хранится в postgresql. Подробная же информация содержит uuid на большую аватарку, и списки наград пользователя и тредов, в которых он является автором. Я НЕ ДОПУСКАЮ запросов вида SELECT id WHERE author=user_id; -- я считаю их слишком медленными.
* [global.rs](https://github.com/TrionProg/server_master/blob/master/src/db/global.rs).
Здесь хранятся общие настройки проекта, например, аватарки по дефолту. Поскольку эта информация часто читается и занимает мало места, она хранится в redis. Поскольку она весьма ветвистая, и тоже часто будет дополняться, она хранится в mongo, причем mongo сохраняет всю историю настроек.
* [web/mod.rs](https://github.com/TrionProg/server_master/blob/master/src/web/mod.rs).
Это простейший web-интерфейс.
