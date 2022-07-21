INSERT INTO public.comment (comment_id, post_id, user_id, content, created_at)
VALUES
    -- from: bob
    ('3a86b8f8-827b-4f14-94a2-34517b4b5bde', 'd9ca2672-24c5-4442-b32f-cd717adffbaa',
     'c994b839-84f4-4509-ad49-59119133d6f5', 'lol bet ur still bad, 1v1 me', '2022-07-29 01:52:31.167673'),
    -- from: alice
    ('d6f862b5-2b87-4af4-b15e-6b3398729e6d', 'd9ca2672-24c5-4442-b32f-cd717adffbaa',
     '51b374f1-93ae-4c5c-89dd-611bda8412ce', 'you''re on!', '2022-07-29 01:53:53.115782'),
    -- from: alice
    ('1eed85ae-adae-473c-8d05-b1dae0a1df63', '7e3d4d16-a35e-46ba-8223-b4f1debbfbfe',
     '51b374f1-93ae-4c5c-89dd-611bda8412ce', 'lol you''re just mad you lost :P', '2022-07-29 01:55:50.116119');

