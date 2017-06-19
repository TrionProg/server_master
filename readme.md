
Цель
----

Основная цель состоит в том, чтобы сделать подобие соц. сети. Данный код мне(TrionProg) пригодится в дальнейшем для некоторых моих проектов.

Требования и особенности:
-------------------------
* Использование тех видов БД, которые наиболее уместны
* Максимальная распределённость, поэтому всё держится на uuid
* Акцент на скорость
* Кэширование с помощью redis
* Развитое API уровня create_user, create_thread
* Сравнительно небольшое количество сложных SQL запросов ввиду того, что в проекте соединены различные БД

Мы разрабатываем от бизнес-логики к БД, а не наоборот, как сказано в задании. Поэтому API первично. Поэтому заполнение всей БД происходит буквально несколькими вызовами API [src/db/generate.rs](https://github.com/TrionProg/server_master/blob/master/src/db/generate.rs).

[Схема БД](https://github.com/TrionProg/server_master/blob/master/DB_diagram.pdf)

В данный момент реализованы:
* [forum.rs](https://github.com/TrionProg/server_master/blob/master/src/db/forum.rs).
Все посты хранятся в cassandra, а для составления списка постов используется таблица posts в postgres, поэтому получение списков постов весьма медленная операция, это временная мера, ибо нужно делать свою БД, которая фактически представляет собой массив id постов. Можно еще и рыться в кассандре, но мы не уверены, что это будет быстым. Посты кэшируются.
* [images.rs](https://github.com/TrionProg/server_master/blob/master/src/db/images.rs).
Это хранилище картинок. Временно ввиду моей глупости они хранятся в postgres, конечно они должны храниться в cassandra. Они так же кэшируются
* [users.rs](https://github.com/TrionProg/server_master/blob/master/src/db/users.rs).
Это большое API для управления пользователями. Информация о пользователе разбита на две части: короткую и полную(эквивалентна стене в соц. сети). Короткая содержит лишь логин, uuid маленькой аватарки и рейтинг. При "сборке" треда форума очень много обращений именно за такой информацией, поэтому эта информация кэшируется путем сериализации в двоичный формат. Постоянно коротка информация хранится в postgresql. Подробная же информация содержит uuid на большую аватарку, и списки наград пользователя и тредов, в которых он является автором. Поскольку при добавлении новых фич пользователя придется часто дописывать, используется mongo. Обратите внимание, что мы НЕ ДОПУСКАЕМ запросов вида SELECT id WHERE author=user_id; мы считаем их слишком медленными.
* [global.rs](https://github.com/TrionProg/server_master/blob/master/src/db/global.rs).
Здесь хранятся общие настройки проекта, например, аватарки по дефолту. Поскольку эта информация часто читается и занимает мало места, она хранится в redis. Поскольку она весьма ветвистая, и тоже часто будет дополняться, она хранится в mongo, причем mongo сохраняет всю историю настроек.
* [web/mod.rs](https://github.com/TrionProg/server_master/blob/master/src/web/mod.rs).
Это простейший веб интерфейс. Посмотреть можно на http://89.110.48.1:8080/threads?catgory=4 , если, конечно, мой системник работает, а электричество не отключили.
* [some_sqls.txt](https://github.com/TrionProg/server_master/blob/master/some_sqls.txt).
Это несколько запросов, используемые при создании бд, функций. Будет расширяться.

Что сделано/что не сделано
--------------------------
* Существуют скомпилированные запросы для cassandra и postgresql, однако для mongo пока что нет
* Еще пока не знакомились с neo4j, но драйвер, фух, к счастью есть https://github.com/livioribeiro/rusted-cypher Я просто создам граф пользователей. Как знаю, neo4j очень медленный, поэтому попробуем сделать лишь 1000.000 связей.
* Хочется сделать таки кластеры, мб хватит времени
* Из CRUD в API почти все CR есть, а вот UD почти нет. Многих D, думаю, не будет, они развалят всю бд. Ну с U проблем нет, я планирую сделать так, чтобы U/D удаляли/модифицировали БД, но кэш бы не трогали, чтобы не было такого как в вк "неизвестная ошибка", когда репостишь удалённый пост например. С очень низкой вероятностью запись в кэше удалится слишком рано. Кстати в кэше у меня живут лишь 30с-2минуты.
* Небольшие отсутствия некоторых вызовов API или таблиц.
* Не особо много оптимизаций БД, во всяком случае где-то в конфигах.
* Ну очень мало сложных запросов, может это и считается нормой для данной работы?! Что-то идей нет.

Ещё мы провели некое тестирование при помощи Jmeter-а. Да, кэширование значительно улучшило ситуацию, правда БД настроены по дефолту и кэш не был особо забит информацией.
[сравнение](https://github.com/TrionProg/server_master/blob/master/caching.png)


Как видите, сделано частично на 4, частично на 5, частично на 3 =)) Но всё-таки мы хотим вытянуть на 4. Почему я(TrionProg) пишу в июне? Потому что я 4 месяца делал [практику](https://github.com/TrionProg/pz5_editor), ибо я всегда указываю как практику/курсач часть проектов и, как всегда, это оказывается в итоге куда сложнее, чем сперва ожидалось. =) В этом проекте и многих зависимостях поблизости 10к строчек всякого матана и алгоритмов. Зато я за 4 месяца сделал то, что не удавалось сделать 4 года, и всё это благодаря Rust. Есть коммиты и поновее, но я решил спрятать исходники, тк не хочу давать конкурентам конкурентное преимущество в виде архитектуры и кода, который можно лишь скачать и переделать.
