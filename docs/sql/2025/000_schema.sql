DROP TABLE IF EXISTS `user`;
CREATE TABLE IF NOT EXISTS `user` (
    `id` INT NOT NULL AUTO_INCREMENT,
    `email` VARCHAR(255) NOT NULL,
    `name` VARCHAR(255) NOT NULL,
    `hashed_password` VARCHAR(255) NOT NULL,

    -- Utility columns
    `status` SMALLINT NOT NULL DEFAULT '1',
    `flag` INT NOT NULL DEFAULT '0',
    `meta` VARCHAR(255),
    `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `created_by` VARCHAR(255),
    `updated_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    `updated_by` VARCHAR(255),
    `deleted_at` TIMESTAMP,
    `deleted_by` VARCHAR(255),
    PRIMARY KEY (`id`),
    UNIQUE KEY `email` (`email`)
) ENGINE = INNODB;

DROP TABLE IF EXISTS `card`;
CREATE TABLE IF NOT EXISTS `card` (
    `id` INT NOT NULL AUTO_INCREMENT,
    `title` VARCHAR(255) NOT NULL,
    `description` VARCHAR(255),
    `card_status` VARCHAR(255),
    `user_id` INT,

    -- Utility columns
    `status` SMALLINT NOT NULL DEFAULT '1',
    `flag` INT NOT NULL DEFAULT '0',
    `meta` VARCHAR(255),
    `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `created_by` VARCHAR(255),
    `updated_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    `updated_by` VARCHAR(255),
    `deleted_at` TIMESTAMP,
    `deleted_by` VARCHAR(255),
    PRIMARY KEY (`id`)
) ENGINE = INNODB;
