INSERT INTO public."user" (user_id, username, password_hash)
VALUES
    -- username: "alice"; password: "rustacean since 2015"
    ('51b374f1-93ae-4c5c-89dd-611bda8412ce', 'alice',
     '$argon2id$v=19$m=4096,t=3,p=1$3v3ats/tYTXAYs3q9RycDw$ZltwjS3oQwPuNmL9f6DNb+sH5N81dTVZhVNbUQzmmVU'),
    -- username: "bob"; password: "pro gamer 1990"
    ('c994b839-84f4-4509-ad49-59119133d6f5', 'bob',
     '$argon2id$v=19$m=4096,t=3,p=1$1zbkRinUH9WHzkyu8C1Vlg$70pu5Cca/s3d0nh5BYQGkN7+s9cqlNxTE7rFZaUaP4c');


