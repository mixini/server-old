# User rules

## all users can read other users
allow(user: User, "read", other_user: User);

## only admins and moderators can update or delete other users
allow(user: User, action, other_user: User) if
    (user.role = "administrator" or user.role = "moderator") and
    (action = "update" or action = "delete");

## users can update or delete themselves
allow(user: User, action, other_user: User) if
    user.id = other_user.id and
    (action = "update" or action = "delete");
